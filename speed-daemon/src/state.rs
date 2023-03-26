pub(crate) use std::collections::HashMap;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::{error, info, warn};
use tokio::sync::mpsc;

use crate::{
    message::{InboundMessageType, OutboundMessageType},
    types::{
        CurrentCameraDb, IssuedTicketsDayDb, Mile, Plate, PlateTimestampCameraDb,
        Road, Speed, TicketDispatcherDb, PlateRoadStruct, TimestampCameraStruct,
    },
};

// Reference: https://github.com/tokio-rs/mini-redis/blob/master/src/db.rs
#[derive(Debug, Clone)]
pub struct Db {
    /// Handle to shared state. The background task will also have an
    /// `Arc<Shared>`.
    shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared {
    /// The shared state is guarded by a mutex. This is a `std::sync::Mutex` and
    /// not a Tokio mutex. This is because there are no asynchronous operations
    /// being performed while holding the mutex. Additionally, the critical
    /// sections are very small.
    ///
    /// A Tokio mutex is mostly intended to be used when locks need to be held
    /// across `.await` yield points. All other cases are **usually** best
    /// served by a std mutex. If the critical section does not include any
    /// async operations but is long (CPU intensive or performing blocking
    /// operations), then the entire operation, including waiting for the mutex,
    /// is considered a "blocking" operation and `tokio::task::spawn_blocking`
    /// should be used.
    state: Mutex<State>,
}

#[derive(Debug)]
struct State {
    dispatchers: TicketDispatcherDb,
    current_camera: CurrentCameraDb,
    plate_timestamp_camera: PlateTimestampCameraDb,
    issued_tickets_day: IssuedTicketsDayDb,
}

impl Db {
    pub fn new() -> Db {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                dispatchers: HashMap::new(),
                current_camera: HashMap::new(),
                // plates_tickets: HashMap::new(),
                plate_timestamp_camera: HashMap::new(),
                issued_tickets_day: HashMap::new(),
            }),
        });
        Db { shared }
    }

    pub fn add_plate_road_timestamp_camera(
        &self,
        plate_road: PlateRoadStruct,
        ts_camera: TimestampCameraStruct,
    ) {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in add_plate_timestamp_camera");

        if let Some(temp_vec) = state.plate_timestamp_camera.get(&plate_road) {
            temp_vec.push(ts_camera);
            state.plate_timestamp_camera.insert(plate_road, *temp_vec);
        }

        
    }

    // This will return a Vec of tickets in a given road where the average speed exceeded the limit between
    // any pair of observations on the same road, even if the observations were not from adjacent cameras.
    pub fn get_tickets_for_plate(&self, plate: &Plate) -> Option<Vec<OutboundMessageType>> {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in get_plate_ts_camera");

        // info!("Processing {}", plate);

        // return immediately if there's only one observation (plate,timestamp)->camera
        if state.plate_timestamp_camera.len() < 2 {
            // info!("Only one observation for plate {}, exiting.", plate);
            return None;
        }

        // this will have all the tickets we need to issue.
        // NOTE: Should only be 1 since we keep track of days on which tickets were issued.
        let mut tickets = Vec::new();

        // Special case of two elements
        if state.plate_timestamp_camera.len() == 2 {
            // info!("Special case of two elements.");

            // "if let Some" to get first & second value from the state.plate_timestamp_camera lose the var context inside if.
            // this is to preserve it for later code.
            let mut pair_vector: Vec<(&PlateTimestamp, &InboundMessageType)> = Vec::new();

            let mut camera_mile1: Mile = 0;
            let mut camera_limit1: Speed = 0;
            let mut camera_mile2: Mile = 0;

            let mut road1: Road = 0;
            let mut road2: Road = 0;

            // get the first value in the hash
            if let Some((p_ts_pair1, camera1)) = state.plate_timestamp_camera.iter().next() {
                info!("First pair: {:?} {:?}", p_ts_pair1, camera1);
                pair_vector.push((p_ts_pair1, camera1));
            }

            // get the second value in the hash
            if let Some((p_ts_pair2, camera2)) = state
                .plate_timestamp_camera
                .iter()
                .nth(1)
                .map(|(k, v)| (k, v))
            {
                // info!("Second pair: {:?} {:?}", p_ts_pair2, camera2);
                pair_vector.push((p_ts_pair2, camera2));
            }

            if let InboundMessageType::IAmCamera { road, mile, limit } = pair_vector[0].1 {
                road1 = *road;
                camera_mile1 = *mile;
                camera_limit1 = *limit;
            } else {
                error!(
                    "Something really bad happened in two element special case, values not found."
                );
            };

            if let InboundMessageType::IAmCamera { road, mile, limit } = pair_vector[1].1 {
                road2 = *road; // road is the same as above
                camera_mile2 = *mile;
                _ = limit;
            } else {
                error!(
                    "Something really bad happened in two element special case, values not found."
                );
            };

            let p_ts_pair1 = pair_vector[0].0;
            let p_ts_pair2 = pair_vector[1].0;

            // make sure we are comparing same plate and same road for avg speed calculations
            if p_ts_pair1.plate == *plate && road1 == road2 {
                // info!(
                //     "Comparing {:?} {:?} \nwith   {:?} {:?}",
                //     p_ts_pair1, pair_vector[0].1, p_ts_pair2, pair_vector[1].1
                // );

                // need to x3600 to convert mi/sec to mi/hr. Later, we'll x100 the actual ticket to comply with the spec.
                let distance_traveled = camera_mile1.abs_diff(camera_mile2) as u32;
                let time_traveled = p_ts_pair1.timestamp.abs_diff(p_ts_pair2.timestamp);
                let average_speed =
                    ((distance_traveled as f32 * 3600.0) / time_traveled as f32).round() as u32; // we must round here

                // info!(
                //     "For plate {} road {} distance {} time traveled {}",
                //     plate, road1, distance_traveled, time_traveled
                // );

                // mile1 and timestamp1 must refer to the earlier of the 2 observations (the smaller timestamp),
                // and mile2 and timestamp2 must refer to the later of the 2 observations (the larger timestamp).
                if p_ts_pair1.timestamp > p_ts_pair2.timestamp {
                    // observation 1 > observation 2, need to swap mile1 & mile2
                    (camera_mile1, camera_mile2) = (camera_mile2, camera_mile1);
                }

                // info!(
                //     "For plate {} between {} {} and {} {} average speed is {} for limit of {}",
                //     plate,
                //     camera_mile1,
                //     p_ts_pair1.timestamp,
                //     camera_mile2,
                //     p_ts_pair2.timestamp,
                //     average_speed,
                //     camera_limit1
                // );

                if average_speed > camera_limit1.into() {
                    let new_ticket = OutboundMessageType::Ticket {
                        plate: plate.clone(),
                        road: road1, //at this point, road1=road2
                        mile1: camera_mile1,
                        timestamp1: p_ts_pair1.timestamp.min(p_ts_pair2.timestamp),
                        mile2: camera_mile2,
                        timestamp2: p_ts_pair1.timestamp.max(p_ts_pair2.timestamp),
                        speed: (average_speed * 100) as Speed,
                    };

                    // Since timestamps do not count leap seconds, days are defined by floor(timestamp / 86400).
                    let day = (p_ts_pair1.timestamp.max(p_ts_pair2.timestamp) as f32 / 86400.0)
                        .floor() as u32;
                    // warn!(
                    //     "Plate {} speed {} exceeded limit {}, preparing {:?} day {}",
                    //     plate, average_speed, camera_limit1, new_ticket, day
                    // );

                    // check if we've previously issued ticket for that day
                    if let Some(check_date) = state.issued_tickets_day.get(plate) {
                        if let Some(_previously_issued_ticket) = check_date.get(&day) {
                            // info!("{} was previously issued a ticket on day {}", plate, day);
                            // return None;
                        } else {
                            {
                                // info!("Plate {} was issued a ticket but not on day {}", plate, day);
                                let mut date_bool_hash = HashMap::new();
                                date_bool_hash.insert(day, true);

                                let mut state = self.shared.state.lock().expect(
                                    "Unable to lock shared state in two element special case",
                                );

                                state
                                    .issued_tickets_day
                                    .insert(plate.clone(), date_bool_hash);

                                tickets.push(new_ticket);
                            }
                        }
                    } else {
                        // info!("Plate {} was never issued a ticket on day {}", plate, day);
                        let mut date_bool_hash = HashMap::new();
                        date_bool_hash.insert(day, true);
                        // info!("Marking day {} as ticketed.", day);

                        state
                            .issued_tickets_day
                            .insert(plate.clone(), date_bool_hash);

                        tickets.push(new_ticket);
                    }
                    // info!("Final tickets db {:?}", tickets);
                }
            }

            // only return the tickets Vec if we have something in it
            if tickets.is_empty() {
                // info!("No tickets found for {}", plate);
                return None;
            } else {
                // info!("Found tickets for {} returning {:?}", plate, tickets);
                return Some(tickets);
            }
        }

        for (p_ts_pair1, camera1) in state.plate_timestamp_camera.clone().iter() {
            for (p_ts_pair2, camera2) in state.plate_timestamp_camera.clone().iter() {
                let mut camera_mile1: Mile = 0;
                let mut camera_limit1: Speed = 0;
                let mut camera_mile2: Mile = 0;

                let mut road1: Road = 0;
                let mut road2: Road = 0;

                // let mut day: u32 = 0;
                // let mut new_ticket: OutboundMessageType = OutboundMessageType::default();

                // We are doing two passes through the same hash, this is value from pass 1
                if let InboundMessageType::IAmCamera { road, mile, limit } = camera1 {
                    road1 = *road;
                    camera_mile1 = *mile;
                    camera_limit1 = *limit;
                } else {
                    error!(
                        "Something really bad happened in get_plate_ts_camera 1, values not found."
                    );
                };

                // We are doing two passes through the same hash, this is value from pass 2
                if let InboundMessageType::IAmCamera { road, mile, limit } = camera2 {
                    road2 = *road;
                    camera_mile2 = *mile;
                    _ = *limit;
                } else {
                    error!(
                        "Something really bad happened in get_plate_ts_camera 2, values not found."
                    );
                };

                // Messages may arrive out of order, so we need to do abs_diff to ensure we don't go negative.
                // Then, make sure we only compare same plate.
                // Then, make sure we don't compare identical timestamps, otherwise div by 0.
                // Then, make sure we only compare different timestamps for the same road
                if p_ts_pair1.plate == p_ts_pair2.plate
                    && p_ts_pair2.plate == *plate
                    && (p_ts_pair1.timestamp != p_ts_pair2.timestamp)
                    && (road1 == road2)
                {
                    // info!(
                    //     "Comparing {:?} {:?} \nwith {:?} {:?}",
                    //     p_ts_pair1, camera1, p_ts_pair2, camera2
                    // );
                    // need to x3600 to convert mi/sec to mi/hr. Later, we'll x100 the actual ticket to comply with the spec.
                    let distance_traveled = camera_mile1.abs_diff(camera_mile2) as u32;
                    let time_traveled = p_ts_pair1.timestamp.abs_diff(p_ts_pair2.timestamp);
                    let average_speed =
                        ((distance_traveled as f32 * 3600.0) / time_traveled as f32).round() as u32; // we must round here

                    // info!(
                    //     "For plate {} road {} distance {} time traveled {}",
                    //     plate, road1, distance_traveled, time_traveled
                    // );

                    // mile1 and timestamp1 must refer to the earlier of the 2 observations (the smaller timestamp),
                    // and mile2 and timestamp2 must refer to the later of the 2 observations (the larger timestamp).
                    if p_ts_pair1.timestamp > p_ts_pair2.timestamp {
                        // observation 1 > observation 2, need to swap mile1 & mile2
                        (camera_mile1, camera_mile2) = (camera_mile2, camera_mile1);
                    }

                    if average_speed > camera_limit1.into() {
                        let new_ticket = OutboundMessageType::Ticket {
                            plate: plate.clone(),
                            road: road1, // road1 = road2 here so we can pick either one
                            mile1: camera_mile1,
                            timestamp1: p_ts_pair1.timestamp.min(p_ts_pair2.timestamp),
                            mile2: camera_mile2,
                            timestamp2: p_ts_pair1.timestamp.max(p_ts_pair2.timestamp),
                            speed: (average_speed * 100) as Speed, //protocol spec requires this to be 100x miles per hour
                        };

                        // Since timestamps do not count leap seconds, days are defined by floor(timestamp / 86400).
                        let day = (p_ts_pair1.timestamp.max(p_ts_pair2.timestamp) as f32 / 86400.0)
                            .floor() as u32;
                        // warn!(
                        //     "Speed {} exceeded limit {}, preparing {:?} day {}",
                        //     average_speed, camera_limit1, new_ticket, day
                        // );

                        // check if we've previously issued ticket for that day
                        if let Some(check_date) = state.issued_tickets_day.get(plate) {
                            if let Some(_previously_issued_ticket) = check_date.get(&day) {
                                // info!("{} was previously issued a ticket on day {}", plate, day);
                                // return None;
                            } else {
                                {
                                    // info!(
                                    //     "Plate {} was issued a ticket but not on day {}",
                                    //     plate, day
                                    // );
                                    let mut date_bool_hash = HashMap::new();
                                    date_bool_hash.insert(day, true);

                                    state
                                        .issued_tickets_day
                                        .insert(plate.clone(), date_bool_hash);

                                    tickets.push(new_ticket);
                                }
                            }
                        } else {
                            // info!("Plate {} was never issued a ticket on day {}", plate, day);
                            let mut date_bool_hash = HashMap::new();
                            date_bool_hash.insert(day, true);

                            state
                                .issued_tickets_day
                                .insert(plate.clone(), date_bool_hash);

                            tickets.push(new_ticket);
                        }
                    }
                }
            }
        }

        // only return the tickets Vec if we have something in it
        if tickets.is_empty() {
            // info!("No tickets found for {}", plate);
            None
        } else {
            // info!("Found tickets for {} returning {:?}", plate, tickets);
            Some(tickets)
        }
    }

    // This is invoked by handle_i_am_camera when a new camera comes online.
    pub fn add_camera(&mut self, addr: SocketAddr, new_camera: InboundMessageType) {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in get_camera");

        state.current_camera.insert(addr, new_camera);
    }

    pub fn get_current_camera(&self, addr: &SocketAddr) -> InboundMessageType {
        let state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in get_current_camera");

        let camera = state
            .current_camera
            .get(addr)
            .expect("Unable to locate camera");
        camera.clone()
    }

    pub fn add_ticket_dispatcher(
        &mut self,
        road: Road,
        addr: SocketAddr,
        tx: mpsc::Sender<OutboundMessageType>,
    ) {
        let mut addr_tx_hash = HashMap::new();
        addr_tx_hash.insert(addr, tx);
        let mut state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in add_ticket_dispatcher");

        state.dispatchers.insert(road, addr_tx_hash);
    }

    pub fn get_ticket_dispatcher(&self, road: Road) -> Option<mpsc::Sender<OutboundMessageType>> {
        let state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in add_ticket_dispatcher");

        // First, we get the hash mapping the road num to the client address-tx hash
        // Second, we get the tx from the client address.
        // NOTE: this overrides the previous ticket dispatcher for the same road. PROBLEM?
        if let Some(addr_tx_hash) = state.dispatchers.get(&road) {
            if let Some((_client_addr, tx)) = addr_tx_hash.iter().next() {
                // info!("Found a dispatcher for road {} at {}", road, client_addr);
                Some(tx.clone())
            } else {
                error!(
                    "BIG PROBLEM, dispatcher was added but somehow not found for road {}!",
                    road
                );
                None
            }
        } else {
            // warn!("No dispatcher found for road {} try again later", road);
            None
        }
    }
}

impl Default for Db {
    fn default() -> Self {
        Self::new()
    }
}

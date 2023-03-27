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
        CurrentCameraDb, Day, IssuedTicketsDayDb, Mile, PlateRoadStruct,
        PlateRoadTimestampCameraDb, Road, Speed, TicketDispatcherDb, TimestampCameraStruct,
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
    plate_road_timestamp_camera: PlateRoadTimestampCameraDb,
    issued_tickets_day: IssuedTicketsDayDb,
}

impl Db {
    pub fn new() -> Db {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                dispatchers: HashMap::new(),
                current_camera: HashMap::new(),
                // plates_tickets: HashMap::new(),
                plate_road_timestamp_camera: HashMap::new(),
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
            .expect("Unable to lock shared state in add_plate_road_timestamp_camera");

        state
            .plate_road_timestamp_camera
            .entry(plate_road)
            .or_default()
            .push(ts_camera);
    }

    // This will return a Vec of tickets in a given road where the average speed exceeded the limit between
    // any pair of observations on the same road, even if the observations were not from adjacent cameras.
    pub fn get_ticket_for_plate(
        &self,
        plate_road: &PlateRoadStruct,
    ) -> Option<OutboundMessageType> {
        fn calculate_average_speed(
            observation1: &TimestampCameraStruct,
            observation2: &TimestampCameraStruct,
        ) -> u32 {
            let mut mile1: Mile = 0;
            let mut mile2: Mile = 0;

            if let InboundMessageType::IAmCamera {
                road: _,
                mile,
                limit: _,
            } = observation1.camera
            {
                mile1 = mile;
            };

            if let InboundMessageType::IAmCamera {
                road: _,
                mile,
                limit: _,
            } = observation2.camera
            {
                mile2 = mile;
            };

            // need to x3600 to convert mi/sec to mi/hr. Later, we'll x100 the actual ticket to comply with the spec.
            let distance_traveled = mile1.abs_diff(mile2) as u32;
            let time_traveled = observation1.timestamp.abs_diff(observation2.timestamp);

            ((distance_traveled as f32 * 3600.0) / time_traveled as f32).round() as u32
        }

        // This returns a tuple of Vec of days where the ticket was generated,
        // plus the ticket. Or None.
        fn generate_ticket(
            observation1: &TimestampCameraStruct,
            observation2: &TimestampCameraStruct,
            plate_road: &PlateRoadStruct,
            average_speed: u32,
        ) -> OutboundMessageType {
            let mut mile1: Mile = 0;
            let mut mile2: Mile = 0;
            let mut common_limit: u16 = 0;

            info!(
                "Between {:?} and {:?} average speed was {}",
                observation1, observation2, average_speed
            );

            if let InboundMessageType::IAmCamera {
                road: _,
                mile,
                limit,
            } = observation1.camera
            {
                mile1 = mile;
                common_limit = limit;
            };

            if let InboundMessageType::IAmCamera {
                road: _,
                mile,
                limit: _,
            } = observation2.camera
            {
                mile2 = mile;
            };

            // mile1 and timestamp1 must refer to the earlier of the 2 observations (the smaller timestamp),
            // and mile2 and timestamp2 must refer to the later of the 2 observations (the larger timestamp).
            let timestamp1 = observation1.timestamp;
            let timestamp2 = observation2.timestamp;

            // mile1 and timestamp1 must refer to the earlier of the 2 observations (the smaller timestamp),
            // and mile2 and timestamp2 must refer to the later of the 2 observations (the larger timestamp).
            if timestamp1 > timestamp2 {
                // observation 1 > observation 2, need to swap mile1 & mile2
                (mile1, mile2) = (mile2, mile1);
            }

            let new_ticket = OutboundMessageType::Ticket {
                plate: plate_road.plate.clone(),
                road: plate_road.road,
                mile1,
                timestamp1: timestamp1.min(timestamp2),
                mile2,
                timestamp2: timestamp1.max(timestamp2),
                speed: (average_speed * 100) as Speed,
            };

            // Return the generated ticket
            new_ticket
        }

        let mut state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in get_plate_ts_camera");

        // For a given (plate,road) combo let's get all the (timestamp, camera) observations in the Vec
        if let Some(vec_of_ts_cameras) = state.plate_road_timestamp_camera.clone().get(plate_road) {
            let mut common_limit = 0;

            if let InboundMessageType::IAmCamera {
                road: _,
                mile: _,
                limit,
            } = vec_of_ts_cameras[0].camera
            {
                common_limit = limit;
            };

            let mut ticket = None;

            match vec_of_ts_cameras.len() {
                0 | 1 => {
                    warn!(
                        "{:?} has fewer than 2 elements in {:?}, no tickets.",
                        plate_road, vec_of_ts_cameras
                    );
                    return None;
                }
                2 => {
                    info!("Special case of 2 entries for {:?}, analyzing.", plate_road);

                    // First, let's calculate the average speed between two observations
                    let average_speed =
                        calculate_average_speed(&vec_of_ts_cameras[0], &vec_of_ts_cameras[1]);

                    // Returns True if none of these days were previously issued a ticket on
                    let mut issue_ticket: bool = true;

                    // calculate the days for both observations
                    let day1 = (vec_of_ts_cameras[0].timestamp as f32 / 86400.0).floor() as u32;
                    let day2 = (vec_of_ts_cameras[1].timestamp as f32 / 86400.0).floor() as u32;

                    if let Some(days) = state.issued_tickets_day.get(plate_road) {
                        for day in days.iter() {
                            // skip if day 1 matches, or
                            // day 2 matches, or
                            //
                            if *day == day1 || *day == day2 {
                                issue_ticket = false;
                            }
                        }

                        if !issue_ticket {
                            warn!("Previously issued tickets for {:?}", days);
                            return None;
                        }
                    }

                    if average_speed > common_limit.into() {
                        let new_ticket = generate_ticket(
                            &vec_of_ts_cameras[0],
                            &vec_of_ts_cameras[1],
                            plate_road,
                            average_speed,
                        );

                        info!(
                            "{:?} ready, sending for dispatch. Day1: {} day2: {}",
                            new_ticket, day1, day2
                        );

                        state
                            .issued_tickets_day
                            .entry(plate_road.to_owned())
                            .or_default()
                            .push(day1);
                        state
                            .issued_tickets_day
                            .entry(plate_road.to_owned())
                            .or_default()
                            .push(day2);

                        ticket = Some(new_ticket);

                        return ticket;
                    }
                }
                _ => {
                    info!("More than 2 entries for {:?}, analyzing.", plate_road);

                    for i in 0..vec_of_ts_cameras.len() {
                        for j in (i + 1)..vec_of_ts_cameras.len() {
                            // First, let's calculate the average speed between two observations
                            let average_speed = calculate_average_speed(
                                &vec_of_ts_cameras[i],
                                &vec_of_ts_cameras[j],
                            );

                            if average_speed > common_limit.into() {
                                let new_ticket = generate_ticket(
                                    &vec_of_ts_cameras[i],
                                    &vec_of_ts_cameras[j],
                                    plate_road,
                                    average_speed,
                                );

                                // Returns True if none of these days were previously issued a ticket on
                                let mut issue_ticket: bool = true;

                                // calculate the days for both observations
                                let day1 = (vec_of_ts_cameras[0].timestamp as f32 / 86400.0).floor()
                                    as u32;
                                let day2 = (vec_of_ts_cameras[1].timestamp as f32 / 86400.0).floor()
                                    as u32;

                                if let Some(days) = state.issued_tickets_day.get(plate_road) {
                                    for day in days.iter() {
                                        // skip if day 1 matches, or
                                        // day 2 matches, or
                                        //
                                        if *day == day1 || *day == day2 {
                                            issue_ticket = false;
                                        }
                                    }

                                    if !issue_ticket {
                                        warn!("Previously issued tickets for {:?}", days);
                                        return None;
                                    }
                                }

                                info!(
                                    "{:?} ready, storing day1: {} day2: {}, dispatching.",
                                    new_ticket, day1, day2
                                );

                                state
                                    .issued_tickets_day
                                    .entry(plate_road.to_owned())
                                    .or_default()
                                    .push(day1);
                                state
                                    .issued_tickets_day
                                    .entry(plate_road.to_owned())
                                    .or_default()
                                    .push(day2);

                                ticket = Some(new_ticket);
                                break;
                            }
                        }
                        if ticket.is_some() {
                            break;
                        }
                    }
                }
            }

            ticket
        } else {
            warn!("No entries found for {:?}, exiting.", plate_road);
            None
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

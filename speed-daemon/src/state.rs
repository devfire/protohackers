pub(crate) use std::collections::HashMap;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::{error, info};
use tokio::sync::mpsc;

use crate::{
    message::{InboundMessageType, OutboundMessageType},
    types::{
        CurrentCameraDb, IssuedTicketsDayDb, Mile, Plate, PlateTicketDb, PlateTimestamp,
        PlateTimestampCameraDb, Road, Speed, TicketDispatcherDb,
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
    plates_tickets: PlateTicketDb,
    plate_timestamp_camera: PlateTimestampCameraDb,
    issued_tickets_day: IssuedTicketsDayDb,
}

impl Db {
    pub fn new() -> Db {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                dispatchers: HashMap::new(),
                current_camera: HashMap::new(),
                plates_tickets: HashMap::new(),
                plate_timestamp_camera: HashMap::new(),
                issued_tickets_day: HashMap::new(),
            }),
        });
        Db { shared }
    }

    pub fn add_plate_timestamp_camera(
        &self,
        plate_timestamp: PlateTimestamp,
        camera: InboundMessageType,
    ) {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in add_plate_timestamp_camera");

        state.plate_timestamp_camera.insert(plate_timestamp, camera);
    }

    // This will return a Vec of tickets in a given road where the average speed exceeded the limit between
    // any pair of observations on the same road, even if the observations were not from adjacent cameras.
    pub fn get_tickets_for_plate(&self, plate: &Plate) -> Option<Vec<OutboundMessageType>> {
        // Immutable borrow for now until later
        let state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in get_plate_ts_camera");

        // return immediately if there's only one observation
        if state.plate_timestamp_camera.len() < 2 {
            return None;
        }

        let mut tickets = Vec::new();

        for (p_ts_pair1, camera1) in state.plate_timestamp_camera.iter() {
            for (p_ts_pair2, camera2) in state.plate_timestamp_camera.iter() {
                let mut camera_mile1: Mile = 0;
                let mut camera_limit1: Speed = 0;
                let mut camera_mile2: Mile = 0;
                let mut camera_limit2: Speed = 0;
                let mut road1: Road = 0; // same as road2
                let mut road2: Road = 0; // same as road1
                let mut day: u32 = 0;

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
                    camera_limit2 = *limit;
                } else {
                    error!(
                        "Something really bad happened in get_plate_ts_camera 2, values not found."
                    );
                };

                // Messages may arrive out of order, so we need to figure out what to subtract from what.
                // Observation 2 > Observation 1, plus make sure the plates match.
                // This check is to ensure we don't add the same entries in reverse order.
                if camera_mile2 > camera_mile1 && p_ts_pair2.plate == *plate {
                    let mut average_speed1: u16 = (camera_mile2.abs_diff(camera_mile1)) * 3600
                        / (p_ts_pair2.timestamp.abs_diff(p_ts_pair1.timestamp)) as Speed;
                    average_speed1 = (average_speed1 as f64 * 100.0).round() as Speed;

                    if average_speed1 > camera_limit1 {
                        info!(
                            "Issuing ticket, plate {} traveled at avg speed {} between {} and {} ts1 {} ts2 {}",
                            plate,
                            average_speed1,
                            camera_mile1,
                            camera_mile2,
                            p_ts_pair2.timestamp,
                            p_ts_pair1.timestamp
                        );
                        let new_ticket = OutboundMessageType::Ticket {
                            plate: plate.clone(),
                            road: road1,
                            mile1: camera_mile1,
                            timestamp1: p_ts_pair1.timestamp,
                            mile2: camera_mile2,
                            timestamp2: p_ts_pair2.timestamp,
                            speed: average_speed1,
                        };

                        if let Some(previously_ticketed_day) = state.issued_tickets_day.get(plate)
                        {
                            // Only add a ticket if it hasn't been issued before
                            if day != *previously_ticketed_day {
                                info!("Adding ticket {:?}", new_ticket);
                                tickets.push(new_ticket);
                            }
                        }
                        // Since timestamps do not count leap seconds, days are defined by floor(timestamp / 86400).
                        day = (p_ts_pair2.timestamp as f32 / 86400.0).floor() as u32;
                    }
                }

                if camera_mile1 > camera_mile2 && p_ts_pair1.plate == *plate {
                    let mut average_speed2: u16 = (camera_mile1.abs_diff(camera_mile2)) * 3600
                        / (p_ts_pair1.timestamp.abs_diff(p_ts_pair2.timestamp)) as Speed;
                    average_speed2 = (average_speed2 as f64 * 100.0).round() as Speed;
                    if average_speed2 > camera_limit2 {
                        info!(
                            "Issuing ticket, plate {} traveled at speed {} between {} and {} ts1 {} ts2 {}",
                            plate,
                            average_speed2,
                            camera_mile2,
                            camera_mile1,
                            p_ts_pair1.timestamp,
                            p_ts_pair2.timestamp
                        );
                        let new_ticket = OutboundMessageType::Ticket {
                            plate: plate.clone(),
                            road: road2,
                            mile1: camera_mile2,
                            timestamp1: p_ts_pair2.timestamp,
                            mile2: camera_mile1,
                            timestamp2: p_ts_pair1.timestamp,
                            speed: average_speed2,
                        };

                        // Get the day if any of a previously issued ticket for the plate
                        if let Some(previously_ticketed_day) = state.issued_tickets_day.get(plate)
                        {
                            // Only add a ticket if it hasn't been issued before
                            if day != *previously_ticketed_day {
                                info!("Adding ticket {:?}", new_ticket);
                                tickets.push(new_ticket);
                            }
                        }

                        // Since timestamps do not count leap seconds, days are defined by floor(timestamp / 86400).
                        day = (p_ts_pair1.timestamp as f32 / 86400.0).floor() as u32;
                    }
                }

                // Borrow it as mutable this time
                let mut state = self
                    .shared
                    .state
                    .lock()
                    .expect("Unable to lock shared state in get_plate_ts_camera");

                state.issued_tickets_day.insert(plate.clone(), day);
            }
        }

        // only return the tickets Vec if we have something in it
        if tickets.is_empty() {
            None
        } else {
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
            if let Some((client_addr, tx)) = addr_tx_hash.iter().next() {
                info!("Found a dispatcher for road {} at {}", road, client_addr);
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

    pub fn add_plate_ticket(&self, plate: Plate, ticket: OutboundMessageType) {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in add_plate_ticket");

        state.plates_tickets.insert(plate, ticket);
    }

    // returns the last ticket for this plate
    pub fn get_plate_ticket(&self, plate: &Plate) -> Option<OutboundMessageType> {
        let state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in add_plate_ticket");

        state.plates_tickets.get(plate).cloned()
    }
}

impl Default for Db {
    fn default() -> Self {
        Self::new()
    }
}

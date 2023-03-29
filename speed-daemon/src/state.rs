pub(crate) use std::collections::HashMap;
use std::{net::SocketAddr, sync::Arc};

use tokio::sync::Mutex;

use log::{error, info, warn};
use tokio::sync::mpsc;

use crate::{
    errors::SpeedDaemonError,
    message::{InboundMessageType, OutboundMessageType},
    types::{
        CurrentCameraDb, IssuedTicketsDayDb, Mile, PlateRoadStruct, PlateRoadTimestampCameraDb,
        Road, Speed, TicketDispatcherDb, TimestampCameraStruct,
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

    pub async fn add_plate_road_timestamp_camera(
        &self,
        plate_road: PlateRoadStruct,
        ts_camera: TimestampCameraStruct,
    ) {
        let mut state = self.shared.state.lock().await;

        // This keeps the array sorted. Should be ok for smaller arrays otherwise BinaryHeap is probably better.
        match state
            .plate_road_timestamp_camera
            .entry(plate_road.clone())
            .or_default()
            .binary_search(&ts_camera)
        {
            Ok(_) => {}
            Err(position) => {
                state
                    .plate_road_timestamp_camera
                    .entry(plate_road.clone())
                    .or_default()
                    .insert(position, ts_camera);
            }
        }
    }

    // This will return a Vec of tickets in a given road where the average speed exceeded the limit between
    // any pair of observations on the same road, even if the observations were not from adjacent cameras.
    pub async fn get_ticket_for_plate(
        &self,
        plate_road: &PlateRoadStruct,
    ) -> Option<Vec<OutboundMessageType>> {
        async fn calculate_average_speed(
            observation1: &TimestampCameraStruct,
            observation2: &TimestampCameraStruct,
        ) -> Result<u32, SpeedDaemonError> {
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

            let average_speed =
                ((distance_traveled as f32 * 3600.0) / time_traveled as f32).round() as u32;

            // info!(
            //     "For {:?} avg speed between {:?} and {:?} was {}",
            //     plate_road, observation1, observation2, average_speed
            // );

            Ok(average_speed)
        }

        let mut state = self.shared.state.lock().await;
        let mut tickets: Vec<OutboundMessageType> = Vec::new();

        if let Some(vec_of_ts_cameras) = state.plate_road_timestamp_camera.clone().get(plate_road) {
            if vec_of_ts_cameras.len() < 2 {
                warn!(
                    "{:?} has fewer than 2 elements in {:?}, no ticket.",
                    plate_road, vec_of_ts_cameras
                );
                return None;
            }

            info!(
                "{:?} has more than 2 elements in {:?}, proceeding.",
                plate_road, vec_of_ts_cameras
            );

            // Master checker loop, looking at consecutive pairs of observations
            for pair in vec_of_ts_cameras.chunks(2) {
                match pair {
                    [first, second] => {
                        info!(
                            "For {:?} comparing {:?} with {:?}",
                            plate_road, first, second
                        );

                        // Get the speed limit, it will be common across both observations since there's only 1 limit per road
                        let mut common_limit = 0;

                        if let InboundMessageType::IAmCamera {
                            road: _,
                            mile: _,
                            limit,
                        } = first.camera
                        {
                            common_limit = limit;
                        };

                        // Then, let's calculate the average speed between two observations
                        let average_speed = calculate_average_speed(first, second)
                            .await
                            .expect("Failed to get average speed");

                        // Calculate the days for both observations
                        let day1 = (first.timestamp as f32 / 86400.0).floor() as u32;
                        let day2 = (second.timestamp as f32 / 86400.0).floor() as u32;

                        info!(
                            "For {:?} timestamp1: {} timestamp2: {} day1: {} day2: {} avg speed: {}",
                            plate_road, first.timestamp, second.timestamp, day1, day2, average_speed
                        );

                        // Assume we are going to create a ticket unless it was already issued this day
                        let mut issue_ticket = true;
                        if let Some(days) = state.issued_tickets_day.get(plate_road) {
                            // check the current second day against every day we've issued tickets before
                            for day in days.iter() {
                                if *day == day1 || *day == day2 {
                                    warn!(
                                    "{:?} was previously issued tickets on day {:?}, no ticket.",
                                    plate_road, day
                                );
                                    // there will be no ticket issued
                                    issue_ticket = false;
                                }
                            }
                        }

                        if average_speed > common_limit.into() && issue_ticket {
                            let mut mile1: Mile = 0;
                            let mut mile2: Mile = 0;

                            info!(
                                "For {:?} between {:?} and {:?} average speed was {}",
                                plate_road, first, second, average_speed
                            );

                            if let InboundMessageType::IAmCamera {
                                road: _,
                                mile,
                                limit: _,
                            } = first.camera
                            {
                                mile1 = mile;
                            };

                            if let InboundMessageType::IAmCamera {
                                road: _,
                                mile,
                                limit: _,
                            } = second.camera
                            {
                                mile2 = mile;
                            };

                            // mile1 and timestamp1 must refer to the earlier of the 2 observations (the smaller timestamp),
                            // and mile2 and timestamp2 must refer to the later of the 2 observations (the larger timestamp).
                            let timestamp1 = first.timestamp;
                            let timestamp2 = second.timestamp;

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

                            state
                                .issued_tickets_day
                                .entry(plate_road.clone())
                                .or_default()
                                .insert(day1);

                            state
                                .issued_tickets_day
                                .entry(plate_road.clone())
                                .or_default()
                                .insert(day2);

                            info!(
                                "{:?} ready, stored day1: {} day2: {}, dispatching.",
                                new_ticket, day1, day2
                            );
                            tickets.push(new_ticket);
                        } else {
                            info!(
                                "For {:?} avg speed was {} but limit was {}, no ticket",
                                plate_road, average_speed, common_limit
                            );
                        }
                    }
                    [first] => info!("For {:?} only {:?} remains", plate_road, first),
                    _ => unreachable!(),
                }
            }
        }

        Some(tickets)
    }

    // This is invoked by handle_i_am_camera when a new camera comes online.
    pub async fn add_camera(&mut self, addr: SocketAddr, new_camera: InboundMessageType) {
        let mut state = self.shared.state.lock().await;

        state.current_camera.insert(addr, new_camera);
    }

    pub async fn get_current_camera(&self, addr: &SocketAddr) -> InboundMessageType {
        let state = self.shared.state.lock().await;

        let camera = state
            .current_camera
            .get(addr)
            .expect("Unable to locate camera");
        camera.clone()
    }

    pub async fn add_ticket_dispatcher(
        &mut self,
        road: Road,
        addr: SocketAddr,
        tx: mpsc::Sender<OutboundMessageType>,
    ) {
        let mut addr_tx_hash = HashMap::new();
        addr_tx_hash.insert(addr, tx);
        let mut state = self.shared.state.lock().await;

        state.dispatchers.insert(road, addr_tx_hash);
    }

    pub async fn get_ticket_dispatcher(
        &self,
        road: Road,
    ) -> Option<mpsc::Sender<OutboundMessageType>> {
        let state = self.shared.state.lock().await;

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

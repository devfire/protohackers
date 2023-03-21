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
        CurrentCameraDb, Plate, PlateTicketDb, PlateTimestamp, PlateTimestampCameraDb, Road, Speed,
        TicketDispatcherDb, Timestamp, TimestampCameraTuple,
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
}

impl Db {
    pub fn new() -> Db {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                dispatchers: HashMap::new(),
                current_camera: HashMap::new(),
                plates_tickets: HashMap::new(),
                plate_timestamp_camera: HashMap::new(),
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
            .expect("Unable to lock shared state in check_cameadd_camera_platera_plate");

        state.plate_timestamp_camera.insert(plate_timestamp, camera);
    }

    // This will return two observations from two cameras in a given road that result in the max speed being exceeded.
    pub fn get_plate_ts_camera(&self, speed: Speed) -> (InboundMessageType, InboundMessageType) {
        todo!()
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

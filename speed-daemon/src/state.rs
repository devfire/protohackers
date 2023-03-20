pub(crate) use std::collections::HashMap;
use std::{
    collections::VecDeque,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::{error, info};
use tokio::sync::mpsc;

use crate::{
    message::{InboundMessageType, OutboundMessageType},
    types::{
        CurrentCameraDb, Plate, PlateCameraDb, PlateCameraTuple, PlateTicketDb, Road,
        TicketDispatcherDb, Timestamp,
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
    plates_cameras: PlateCameraDb,
    plates_tickets: PlateTicketDb,
}

impl Db {
    pub fn new() -> Db {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                dispatchers: HashMap::new(),
                current_camera: HashMap::new(),
                plates_cameras: HashMap::new(),
                plates_tickets: HashMap::new(),
            }),
        });
        Db { shared }
    }

    // This function returns a previously seen Plate -> (Timestamp, Camera) mapping
    // Need this because when a camera reports a plate, we don't know if this is a first sighting of that plate or not.
    // So we need to keep track of plates and whether any camera has seen it previously.
    pub fn check_camera_plate(&self, plate: &Plate) -> Option<PlateCameraTuple> {
        let state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in check_camera_plate");

        state.plates_cameras.get(plate).cloned()
    }

    pub fn add_camera_plate(&self, plate: Plate, timestamp: Timestamp, camera: InboundMessageType) {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in check_cameadd_camera_platera_plate");

        state.plates_cameras.insert(plate, (timestamp, camera));
    }

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

    pub fn add_plate_ticket(&self, plate: Plate, ticket: InboundMessageType) {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in add_plate_ticket");

        state.plates_tickets.insert(plate, ticket);
    }

    // returns the last ticket for this plate
    pub fn get_plate_ticket(&self, plate: Plate) -> Option<InboundMessageType> {
        let state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in add_plate_ticket");

        if let Some(ticket) = state.plates_tickets.get(&plate) {
            Some(ticket.clone())
        } else {
            None
        }
    }
}

impl Default for Db {
    fn default() -> Self {
        Self::new()
    }
}

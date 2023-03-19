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
    types::{CurrentCameraDb, PlateCameraDb, Road, TicketDispatcherDb, TicketQueue},
};

// Reference: https://github.com/tokio-rs/mini-redis/blob/master/src/db.rs
#[derive(Debug, Clone)]
pub(crate) struct Db {
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
    ticket_queue: TicketQueue,
}

impl Db {
    pub fn new() -> Db {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                dispatchers: HashMap::new(),
                current_camera: HashMap::new(),
                plates_cameras: HashMap::new(),
                ticket_queue: VecDeque::new(),
            }),
        });
        Db { shared }
    }

    pub fn add_camera(&mut self, addr: SocketAddr, new_camera: InboundMessageType) {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in get_camera");
        // self.current_camera.insert(addr, new_camera);
        state.current_camera.insert(addr, new_camera);
    }

    pub fn get_current_camera(&self, addr: &SocketAddr) -> InboundMessageType {
        let state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in get_current_camera");
        // let camera = self
        //     .current_camera
        //     .get(addr)
        //     .expect("Unable to locate camera for client");
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
        let addr_tx_hash = state
            .dispatchers
            .get(&road)
            .expect("Unable to fetch addr_tx_hash");

        if let Some((client_addr, tx)) = addr_tx_hash.iter().next() {
            info!("Found a dispatcher for road {} at {}", road, client_addr);
            Some(tx.clone())
        } else {
            error!("Dispatcher for road {} not found", road);
            None
        }
    }

    // Checks to see if there's a ticket in the queue. If there is, returns the ticket.
    // If not, returns None.
    pub fn get_ticket(&mut self) -> Option<OutboundMessageType> {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in get_ticket");

        if let Some(ticket) = state.ticket_queue.pop_front() {
            info!("Found a ticket {:?}", ticket);
            Some(ticket)
        } else {
            None
        }
    }

    // Append a ticket to the shared queue. Once a dispatcher comes online, it'll be delivered.
    pub fn add_ticket(&mut self, new_ticket: OutboundMessageType) {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("Unable to lock shared state in add_ticket");

        state.ticket_queue.push_back(new_ticket);
    }
}

impl Default for Db {
    fn default() -> Self {
        Self::new()
    }
}

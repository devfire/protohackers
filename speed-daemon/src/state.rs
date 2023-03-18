pub(crate) use std::collections::HashMap;
use std::{collections::VecDeque, net::SocketAddr};

use log::{error, info};
use tokio::sync::mpsc;

use crate::{
    message::{InboundMessageType, OutboundMessageType},
    types::{CurrentCameraDb, PlateCameraDb, Road, TicketDispatcherDb, TicketQueue},
};

pub struct SharedState {
    pub dispatchers: TicketDispatcherDb,
    pub current_camera: CurrentCameraDb,
    pub plates_cameras: PlateCameraDb,
    pub ticket_queue: TicketQueue,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            dispatchers: HashMap::default(),
            current_camera: HashMap::default(),
            plates_cameras: HashMap::default(),
            ticket_queue: VecDeque::default(),
        }
    }

    pub fn add_camera(&mut self, addr: SocketAddr, new_camera: InboundMessageType) {
        self.current_camera.insert(addr, new_camera);
    }

    pub fn get_current_camera(&self, addr: &SocketAddr) -> &InboundMessageType {
        let camera = self
            .current_camera
            .get(addr)
            .expect("Unable to locate camera for client");

        camera
    }

    pub fn add_ticket_dispatcher(
        &mut self,
        road: Road,
        addr: SocketAddr,
        tx: mpsc::Sender<OutboundMessageType>,
    ) {
        let mut addr_tx_hash = HashMap::new();
        addr_tx_hash.insert(addr, tx);
        self.dispatchers.insert(road, addr_tx_hash);
    }

    pub fn get_ticket_dispatcher(&self, road: Road) -> Option<&mpsc::Sender<OutboundMessageType>> {
        // First, we get the hash mapping the road num to the client address-tx hash
        // Second, we get the tx from the client address.
        // NOTE: this overrides the previous ticket dispatcher for the same road. PROBLEM?
        let addr_tx_hash = self
            .dispatchers
            .get(&road)
            .expect("Unable to fetch addr_tx_hash");

        if let Some((client_addr, tx)) = addr_tx_hash.iter().next() {
            info!("Found a dispatcher for road {} at {}", road, client_addr);
            Some(tx)
        } else {
            error!("Dispatcher for road {} not found", road);
            None
        }
    }

    // Checks to see if there's a ticket in the queue. If there is, returns the ticket.
    // If not, returns None.
    pub fn get_ticket(&mut self) -> Option<OutboundMessageType> {
        if let Some(ticket) = self.ticket_queue.pop_front() {
            info!("Found a ticket {:?}", ticket);
            Some(ticket)
        } else {
            None
        }
    }

    // Append a ticket to the shared queue. Once a dispatcher comes online, it'll be delivered.
    pub fn add_ticket(&mut self, new_ticket: OutboundMessageType) {
        self.ticket_queue.push_back(new_ticket);
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}

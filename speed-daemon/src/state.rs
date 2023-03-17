pub(crate) use std::collections::HashMap;
use std::net::SocketAddr;

use tokio::sync::mpsc;

use crate::{
    message::{InboundMessageType, OutboundMessageType},
    types::{CurrentCameraDb, PlateCameraDb, Road, TicketDispatcherDb},
};

pub struct SharedState {
    pub dispatchers: TicketDispatcherDb,
    pub current_camera: CurrentCameraDb,
    pub plates_cameras: PlateCameraDb,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            dispatchers: HashMap::default(),
            current_camera: HashMap::default(),
            plates_cameras: HashMap::default(),
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

    pub fn get_ticket_dispatcher(
        &self,
        road: Road,
        addr: &SocketAddr,
    ) -> &mpsc::Sender<OutboundMessageType> {
        // First, we get the hash mapping the road num to the client address-tx hash
        // Second, we get the tx from the client address
        let addr_tx_hash = self
            .dispatchers
            .get(&road)
            .expect("Unable to fetch addr_tx_hash");
        let tx = addr_tx_hash
            .get(addr)
            .expect("Unable to get dispatcher tx from addr_tx_hash");
        tx
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}

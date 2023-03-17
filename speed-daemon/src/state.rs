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
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}

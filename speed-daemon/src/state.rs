pub(crate) use std::collections::HashMap;

use crate::{
    message::InboundMessageType,
    types::{PlateCameraDb, TicketDispatcherDb},
};

pub struct SharedState {
    pub dispatchers: TicketDispatcherDb,
    pub current_camera: InboundMessageType,
    pub plates_cameras: PlateCameraDb,
}

impl SharedState {
    pub fn new(
        dispatchers: TicketDispatcherDb,
        current_camera: InboundMessageType,
        plates_cameras: PlateCameraDb,
    ) -> Self {
        Self {
            dispatchers: HashMap::default(),
            current_camera: InboundMessageType::default(),
            plates_cameras: HashMap::default(),
        }
    }

    pub fn add_camera(&mut self, new_camera: InboundMessageType) {
        self.current_camera = new_camera;
    }
}

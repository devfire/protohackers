use crate::types::{Mile, Plate, Road, Timestamp};

#[derive(Clone, Debug)]
pub enum InboundMessageType {
    Plate { plate: Plate, timestamp: Timestamp },
    WantHeartbeat { interval: u32 },
    IAmCamera { road: Road, mile: Mile, limit: u16 },
    IAmDispatcher { numroads: u8, roads: Vec<u16> },
}

impl InboundMessageType {
    pub fn new_plate(plate: String, timestamp: u32) -> Self {
        InboundMessageType::Plate { plate, timestamp }
    }

    pub fn new_want_heartbeat(interval: u32) -> Self {
        InboundMessageType::WantHeartbeat { interval }
    }

    pub fn new_i_am_camera(road: u16, mile: u16, limit: u16) -> Self {
        InboundMessageType::IAmCamera { road, mile, limit }
    }

    pub fn new_i_am_dispatcher(numroads: u8, roads: Vec<u16>) -> Self {
        InboundMessageType::IAmDispatcher { numroads, roads }
    }
}

impl Default for InboundMessageType {
    fn default() -> Self {
        InboundMessageType::IAmCamera {
            road: 0,
            mile: 0,
            limit: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum OutboundMessageType {
    Heartbeat,

    //0x10: Error (Server->Client)
    Error(String),

    Ticket {
        plate: String,
        road: u16,
        mile1: u16,
        timestamp1: u32,
        mile2: u16,
        timestamp2: u32,
        speed: u16,
    },
}

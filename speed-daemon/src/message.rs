use crate::types::{Mile, Plate, Road, Timestamp};

#[derive(Clone, Debug)]
pub enum InboundMessageType {
    Plate { plate: Plate, timestamp: Timestamp },
    WantHeartbeat { interval: u32 },
    IAmCamera { road: Road, mile: Mile, limit: u16 },
    IAmDispatcher { roads: Vec<u16> },
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

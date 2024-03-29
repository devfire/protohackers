use crate::types::{Mile, Plate, Road, Speed, Timestamp};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

#[derive(Clone, Debug)]
pub enum OutboundMessageType {
    Heartbeat,

    //0x10: Error (Server->Client)
    Error(String),

    Ticket {
        plate: Plate,
        road: Road,
        mile1: Mile,
        timestamp1: Timestamp,
        mile2: Mile,
        timestamp2: Timestamp,
        speed: Speed,
    },
}

impl Default for OutboundMessageType {
    fn default() -> Self {
        OutboundMessageType::Ticket {
            plate: String::from(""),
            road: 0,
            mile1: 0,
            timestamp1: 0,
            mile2: 0,
            timestamp2: 0,
            speed: 0,
        }
    }
}

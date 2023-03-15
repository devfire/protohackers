#[derive(Clone, Debug)]
pub enum InboundMessageType {
    Plate {
        plate: String,
        timestamp: u32,
    },
    WantHeartbeat {
        interval: u32,
    },
    IAmCamera {
        road: u16,
        mile: u16,
        limit: u16,
    },
    IAmDispatcher {
        numroads: u8,
        roads: Vec<u16>,
    },
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
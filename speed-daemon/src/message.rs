use bytes::{BytesMut, BufMut};

#[derive(Clone, Debug)]
pub enum InboundMessageType {
    Plate {
        plate: String,
        timestamp: u32,
    },
    Ticket {
        plate: String,
        road: u16,
        mile1: u16,
        timestamp1: u32,
        mile2: u16,
        timestamp2: u32,
        speed: u16,
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OutboundMessageType {
    Heartbeat,
}

impl OutboundMessageType {
    pub fn convert_to_bytes (&self, dst: &mut BytesMut) {
        match self {
            // 0x41: Heartbeat (Server->Client)
            OutboundMessageType::Heartbeat => {
                dst.put_u8(0x41);
            }
        }
    }
}
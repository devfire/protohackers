use crate::errors::SpeedDaemonError;

use nom::IResult;
use nom::number::complete::{be_u16, be_u8};
use nom::multi::length_data;
use nom::bytes::complete::tag;


#[derive(Clone, Debug)]
pub enum MessageType {
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
    Heartbeat,
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

impl MessageType {
    fn take_str(s: &[u8]) -> IResult<&[u8], &[u8]> {
        length_data(be_u8)(s)
      }

    fn take_timestamp(input: &str) -> IResult<&str, u32> {
        be_u16(input)
    }

    pub fn parse_plate(input: &str) -> IResult<&str, MessageType> {
        let (input, (plate, timestamp)) = (take_str, take_timestamp).parse(input)?;

        Ok((input, MessageType::Plate{plate,timestamp}))
    }
}

// impl MessageType {
//     // returns client->server message type.
//     // NOTE: message types not listed are server->client types.
//     pub fn get_message_type(msg_id: u8) -> Result<Self, SpeedDaemonError> {
//         match msg_id {
//             0x20 => Self::Plate::parse
//             0x40 => Ok(Self::WantHeartbeat),
//             0x80 => Ok(Self::IAmCamera),
//             0x81 => Ok(Self::IAmDispatcher),
//             _ => Err(SpeedDaemonError::InvalidMessage),
//         }
//     }
// }

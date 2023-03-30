use log::info;
use tokio_util::codec::{Decoder, Encoder};

use bytes::{Buf, BufMut, BytesMut};
use nom::{Err, Needed};
// use std::{cmp, fmt, io, str, usize};

use crate::{
    errors::SpeedDaemonError,
    message::{InboundMessageType, OutboundMessageType},
    parsers::parse_message,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MessageCodec {}

impl MessageCodec {
    /// Creates a new [`MessageCodec`].
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MessageCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for MessageCodec {
    //NOTE: #[from] std::io::Error is required in the error definition
    type Error = SpeedDaemonError;

    type Item = InboundMessageType;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }
        match parse_message(src) {
            Ok((remaining_bytes, parsed_message)) => {
                // advance the cursor by the difference between what we read
                // and what we parsed
                src.advance(src.len() - remaining_bytes.len());

                // return the parsed message
                Ok(Some(parsed_message))
            }
            Err(Err::Incomplete(Needed::Size(_))) => Ok(None),
            Err(_) => Err(SpeedDaemonError::ParseFailure),
        }
    }
}

impl Encoder<OutboundMessageType> for MessageCodec {
    type Error = SpeedDaemonError;

    fn encode(&mut self, item: OutboundMessageType, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            OutboundMessageType::Heartbeat => {
                dst.reserve(1);
                dst.put_u8(0x41);
            }

            OutboundMessageType::Error(err_msg) => {
                // 1 byte for the message ID, 1 byte for the string length.
                // Remaining for the length of error message
                let buffer_size = 1 + 1 + err_msg.len();
                dst.reserve(buffer_size);

                dst.put_u8(0x10); // msg type
                dst.put_u8(err_msg.len() as u8); // string length
                dst.put(err_msg.as_bytes()); // error message
                // info!("Encoded error message: {:#?}", dst);
            }

            OutboundMessageType::Ticket {
                plate,
                road,
                mile1,
                timestamp1,
                mile2,
                timestamp2,
                speed,
            } => {
                let buffer_size = 1 + 1 + plate.len() + 2 + 2 + 4 + 2 + 4 + 2;
                let plate = plate.as_bytes();
                dst.reserve(buffer_size);
                dst.put_u8(0x21);

                dst.put_u8(plate.len() as u8);
                dst.extend_from_slice(plate);
                dst.put_u16(road);
                dst.put_u16(mile1);
                dst.put_u32(timestamp1);
                dst.put_u16(mile2);
                dst.put_u32(timestamp2);
                dst.put_u16(speed);
            }
        }

        Ok(())
    }
}

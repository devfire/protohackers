use std::io;

// use anyhow::Error;
use log::{error, info};
use tokio_util::codec::{Decoder, Encoder};

use bytes::{Buf, BufMut, BytesMut};
use nom::{Err, Needed};

use crate::{errors::LRCPError, message::MessageType, parser::parse_message};

#[derive(Clone, Debug, Hash)]
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
    type Item = MessageType;
    type Error = LRCPError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, LRCPError> {
        if src.is_empty() {
            return Ok(None);
        }
        // info!("Decoding {:?}", src);

        match parse_message(src) {
            Ok((_remaining_bytes, parsed_message)) => {
                // advance the cursor by the difference between what we read
                // and what we parsed
                // Whatever we get is all we get, clear the buffer post parsing.
                // src.advance(src.len() - remaining_bytes.len());

                // info!("parse_message: {parsed_message:?}");
                src.clear();

                // return the parsed message
                Ok(Some(parsed_message))
            }
            Err(Err::Incomplete(Needed::Size(_))) => {
                src.clear();
                Ok(None)
            }

            Err(nom::Err::Incomplete(_)) => {
                src.clear();
                Ok(None)
            }

            Err(_) => {
                src.clear();
                Err(LRCPError::ParseFailure)
            }
        }
    }
}

impl Encoder<MessageType> for MessageCodec {
    type Error = io::Error;

    fn encode(&mut self, item: MessageType, dst: &mut BytesMut) -> Result<(), Self::Error> {
        info!("Encoding {item:?}");
        match item {
            MessageType::Connect { session } => todo!(),
            MessageType::Ack { session, length } => {
                // /ack/SESSION/LENGTH/
                let prefix = "/ack/".as_bytes();

                // forward slash, ack, forward slash, session, forward slash, length, forward slash
                let buffer_size = 1 + 3 + 1 + 4 + 1 + 4 + 1;

                dst.reserve(buffer_size);

                let session_str = session.to_string();
                let length_str = length.to_string();

                dst.extend_from_slice(prefix);
                dst.put(session_str.as_bytes());
                dst.put("/".as_bytes());
                dst.put(length_str.as_bytes());
                dst.put("/".as_bytes());
            }
            MessageType::Data { session, pos_data } => todo!(),
            MessageType::Close { session } => todo!(),
        }
        Ok(())
    }
}

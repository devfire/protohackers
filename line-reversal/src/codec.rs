use std::io;

use anyhow::Error;
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
        info!("Decoding {:?}", src);

        match parse_message(src) {
            Ok((_remaining_bytes, parsed_message)) => {
                // advance the cursor by the difference between what we read
                // and what we parsed
                // src.advance(src.len() - remaining_bytes.len());

                info!("parse_message: {parsed_message:?}");
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

            Err(_) => Err(LRCPError::ParseFailure),
        }
    }
}

impl Encoder<MessageType> for MessageCodec {
    type Error = io::Error;

    fn encode(&mut self, item: MessageType, dst: &mut BytesMut) -> Result<(), Self::Error> {
        info!("Encoding {item:?}");
        Ok(())
    }
}

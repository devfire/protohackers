use std::io;

use anyhow::Error;
use log::{error, info};
use tokio_util::codec::{Decoder, Encoder};

use bytes::{Buf, BufMut, BytesMut};
use nom::{Err, Needed};

use crate::{message::MessageType, parser::parse_message};

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
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {
        if src.is_empty() {
            return Ok(None);
        }
        info!("Decoding {:?}", src);

        match parse_message(src) {
            Ok((remaining_bytes, parsed_message)) => {
                // advance the cursor by the difference between what we read
                // and what we parsed
                src.advance(src.len() - remaining_bytes.len());

                info!("parse_message: {parsed_message:?}");

                // return the parsed message
                Ok(Some(parsed_message))
            }
            Err(Err::Incomplete(Needed::Size(_))) => Ok(None),

            Err(nom::Err::Incomplete(_)) => Ok(None),

            Err(nom::Err::Error(nom::error::Error { code, .. }))
            | Err(nom::Err::Failure(nom::error::Error { code, .. })) => {
                error!("Parse error {src:?}");
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("{code:?} during parsing of {src:?}"),
                ));
            }
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

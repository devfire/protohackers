use tokio_util::codec::{Decoder, Encoder};

use bytes::{Buf, BytesMut};
use nom::{Err, Needed};
// use std::{cmp, fmt, io, str, usize};

use crate::{errors::SpeedDaemonError, message::InboundMessageType, parsers::parse_message};

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

use tokio_util::codec::{Decoder, Encoder};

use bytes::{Buf, BufMut, BytesMut};
use std::{cmp, fmt, io, str, usize};

use crate::{errors::SpeedDaemonError, message::MessageType};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MessageCodec {}

impl MessageCodec {
    /// Creates a new [`MessageCodec`].
    pub fn new() -> Self { Self {  } }

    
}

impl Decoder for MessageCodec {
    //NOTE: #[from] std::io::Error is required in the error definition
    type Error = SpeedDaemonError;

    type Item = MessageType;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // if src.is_empty() {
        //     return Ok(None);
        // }

        
    }
}

impl Default for MessageCodec {
    fn default() -> Self {
        Self::new()
    }
}


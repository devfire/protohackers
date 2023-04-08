use bytes::Bytes;

use crate::types::{Session, Length, Pos};

#[derive(Clone, Debug)]
pub enum MessageType {
    Connect {
        session: Session,
    },
    Ack {
        session: Session,
        length: Length,
    },
    Data {
        session: Session,
        pos: Pos,
        data: Bytes,
    },
    Close {
        session: Session,
    },
}

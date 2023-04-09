use crate::types::{Length, SessionPosDataStruct, Session};

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
        pos_data: SessionPosDataStruct,
    },
    Close {
        session: Session,
    },
}

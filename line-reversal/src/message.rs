use crate::types::{Length, PosDataStruct, Session};

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
        pos_data: PosDataStruct,
    },
    Close {
        session: Session,
    },
}

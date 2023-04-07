use bytes::Bytes;

type Session = u32;
type Pos = u32;
type Length = u32;

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

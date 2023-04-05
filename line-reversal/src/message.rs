use bytes::Bytes;

type Session = u32;
type Pos = u32;
type Length = u32;

pub enum MessageType {
    Connect { session: Session },
    Ack(AckStruct),
    Data(DataStruct),
    Close { session: Session },
}

pub struct AckStruct {
    session: Session,
    length: Length,
}

pub struct DataStruct {
    session: Session,
    pos: Pos,
    data: Bytes,
}
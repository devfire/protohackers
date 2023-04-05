use bytes::Bytes;

type Session = u32;
type Pos = u32;
type Length = u32;

#[derive(Clone, Debug)]
pub enum MessageType {
    Connect { session: Session },
    Ack(AckStruct),
    Data(DataStruct),
    Close { session: Session },
}

#[derive(Clone, Debug)]
pub struct AckStruct {
    pub session: Session,
    pub length: Length,
}
#[derive(Clone, Debug)]

pub struct DataStruct {
    pub session: Session,
    pub pos: Pos,
    pub data: Bytes,
}
use std::{collections::HashMap, net::SocketAddr};

// use bytes::{Bytes, BytesMut};

pub type Session = u32;
pub type Pos = u32;
pub type Length = u32;

#[derive(Default, Debug, Clone)]
pub struct SessionPosDataStruct {
    pub session: Session,
    pub pos: Pos,
    pub data: String,
}

impl SessionPosDataStruct {
    /// Creates a new [`SessionPosDataStruct`].
    pub fn new(session: Session, pos: Pos, data: String) -> Self {
        Self { session, pos, data }
    }
}

pub type SocketAddrSessionDb = HashMap<SocketAddr, SessionPosDataStruct>;

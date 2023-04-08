use std::{collections::HashMap, net::SocketAddr};

use bytes::{Bytes, BytesMut};

pub type Session = u32;
pub type Pos = u32;
pub type Length = u32;

#[derive(Debug, Clone)]
pub struct PosDataStruct {
    pos: Pos,
    data: BytesMut,
}

impl PosDataStruct {
    /// Creates a new [`PosDataStruct`].
    pub fn new(pos: Pos, data: BytesMut) -> Self {
        Self { pos, data }
    }
}


pub type SocketAddrSessionDb = HashMap<SocketAddr, HashMap<Session, PosDataStruct>>;

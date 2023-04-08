use std::{collections::HashMap, net::SocketAddr};

pub type Session = u32;
pub type Pos = u32;
pub type Length = u32;

pub type SocketAddrSessionDb = HashMap<SocketAddr, HashMap<Session,Pos>>;
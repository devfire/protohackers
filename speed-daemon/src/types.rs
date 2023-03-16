use std::{
    collections::HashMap,
    net::SocketAddr,
};

use tokio::sync::mpsc;

use crate::message::{InboundMessageType, OutboundMessageType};

pub type Road = u16;
pub type Mile = u16;
pub type Timestamp = u32;
pub type Speed = u16;
pub type Plate = String;

// ----------------Shared state data structures----------------
// A hash of Plate -> (timestamp, IAmCamera)
pub type PlateCameraDb = HashMap<Plate, (Timestamp, InboundMessageType)>;
pub type TicketDispatcherDb = HashMap<Road, HashMap<SocketAddr, mpsc::Sender<OutboundMessageType>>>;
// ------------------------------------------------------------

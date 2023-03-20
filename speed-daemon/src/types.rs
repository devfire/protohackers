use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
};

use tokio::sync::mpsc;

use crate::message::{InboundMessageType, OutboundMessageType};

pub type Road = u16;
pub type Mile = u16;
pub type Timestamp = u32;
pub type Speed = u16;
pub type Plate = String;
pub type PlateCameraTuple = (Timestamp,InboundMessageType);

// ----------------Shared state data structures----------------
// A hash of Plate -> (timestamp, IAmCamera)
pub type PlateCameraDb = HashMap<Plate, PlateCameraTuple>;

// This maps a road ID to a hash of IP,tx
pub type TicketDispatcherDb = HashMap<Road, HashMap<SocketAddr, mpsc::Sender<OutboundMessageType>>>;

// Since tickets can be issued before a ticket dispatcher connects,
// we need a way to store the tickets until then.
pub type TicketQueue = VecDeque<OutboundMessageType>;

// This stores the current tokio task camera. Once a new plate is received,
// we need to check the camera's mile marker and speed limit to calculate the avg speed.
pub type CurrentCameraDb = HashMap<SocketAddr, InboundMessageType>;
// ------------------------------------------------------------

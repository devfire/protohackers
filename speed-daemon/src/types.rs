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
pub type PlateCameraTuple = (Timestamp,InboundMessageType);

// ----------------Shared state data structures----------------
// A hash of Plate -> (timestamp, IAmCamera)
pub type PlateCameraDb = HashMap<Plate, PlateCameraTuple>;

// This maps a road ID to a hash of IP,tx
pub type TicketDispatcherDb = HashMap<Road, HashMap<SocketAddr, mpsc::Sender<OutboundMessageType>>>;

// Since we don't allow more than one ticket pre day we need to store the tickets.
// This is a hash of Plate -> Ticket. The ticket includes the timestamp so we can calculate when the last ticket was issued.
pub type PlateTicketDb = HashMap<Plate, OutboundMessageType>;

// This stores the current tokio task camera. Once a new plate is received,
// we need to check the camera's mile marker and speed limit to calculate the avg speed.
pub type CurrentCameraDb = HashMap<SocketAddr, InboundMessageType>;
// ------------------------------------------------------------

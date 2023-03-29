use std::{collections::HashMap, hash::Hash, net::SocketAddr, cmp::Ordering};

use tokio::sync::mpsc;

use crate::message::{InboundMessageType, OutboundMessageType};

pub type Road = u16;
pub type Mile = u16;
pub type Limit = u16;
pub type Timestamp = u32;
pub type Speed = u16;
pub type Plate = String;
pub type Day = u32;

#[derive(Clone, Debug, Eq, Hash)]
pub struct TimestampCameraStruct {
    pub timestamp: Timestamp,
    pub camera: InboundMessageType,
}

impl PartialOrd for TimestampCameraStruct {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimestampCameraStruct {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl PartialEq for TimestampCameraStruct {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}


#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PlateRoadStruct {
    pub plate: Plate,
    pub road: Road,
}

impl PlateRoadStruct {
    pub fn new(plate: Plate, road: Road) -> Self {
        Self { plate, road }
    }
}

// ----------------Shared state data structures----------------
// A hash of Plate -> (timestamp, IAmCamera)
// pub type PlateCameraDb = HashMap<Plate, TimestampCameraTuple>;

// This contains the (Plate,Timestamp) -> Camera mapping
// pub type PlateTimestampCameraDb = HashMap<PlateTimestamp, InboundMessageType>;

// This contains the Plate -> Vec<Timestamp,Camera> mapping
pub type PlateRoadTimestampCameraDb = HashMap<PlateRoadStruct, Vec<TimestampCameraStruct>>;

// This maps a road ID to a hash of IP,tx
pub type TicketDispatcherDb = HashMap<Road, HashMap<SocketAddr, mpsc::Sender<OutboundMessageType>>>;

// Since we don't allow more than one ticket pre day we need to store the tickets.
// This is a hash of Plate -> Ticket. The ticket includes the timestamp so we can calculate when the last ticket was issued.
pub type PlateTicketDb = HashMap<Plate, OutboundMessageType>;

// This stores the current tokio task camera. Once a new plate is received,
// we need to check the camera's mile marker and speed limit to calculate the avg speed.
pub type CurrentCameraDb = HashMap<SocketAddr, InboundMessageType>;

// This keeps a mapping Plate to a Vec of days, where days are defined by floor(timestamp / 86400)
// i.e. a plate "FOO" could have been ticketed on multiple days
// Every day that contributed to a ticket gets stored.
pub type IssuedTicketsDayDb = HashMap<PlateRoadStruct, Day>;
// ------------------------------------------------------------

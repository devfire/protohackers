use std::{sync::{Arc, Mutex}, collections::HashMap};

use crate::message::{InboundMessageType, OutboundMessageType};

pub type Road = u16;
pub type Mile = u16;
pub type Timestamp = u32;
pub type Speed = u16;
pub type Plate = String;

// ----------------Shared state data structures----------------
// A hash of Plate -> (timestamp, IAmCamera)
pub type Db = Arc<Mutex<HashMap<String, (u32, InboundMessageType)>>>;
pub type TicketDispatcherDb =
    Arc<Mutex<HashMap<u16, Vec<tokio::sync::mpsc::Sender<OutboundMessageType>>>>>;
// ------------------------------------------------------------
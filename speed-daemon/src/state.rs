use std::{
    collections::HashMap,
    net::{SocketAddr},
};

use tokio::sync::mpsc;

use crate::{types::Road, message::OutboundMessageType};


pub struct State {
    dispatchers : HashMap<Road, HashMap<SocketAddr, mpsc::Sender<OutboundMessageType>>>,
}
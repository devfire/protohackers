use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::{Framed, LinesCodec};

/// Shorthand for the transmit half of the message channel.
pub type Tx = mpsc::UnboundedSender<String>;

/// Shorthand for the receive half of the message channel.
pub type Rx = mpsc::UnboundedReceiver<String>;

#[derive(Debug)]
pub struct UserDetails {
    username: String,
    tx: Tx,
}
/// Data that is shared between all peers in the chat server.
///
/// This is the set of `Tx` handles for all connected clients. Whenever a
/// message is received from a client, it is broadcasted to all peers by
/// iterating over the `peers` entries and sending a copy of the message on each
/// `Tx`.
pub struct Shared {
    pub peers: HashMap<SocketAddr, UserDetails>,
}

/// The state for each connected client.
pub struct Peer {
    /// The TCP socket wrapped with the `Lines` codec, defined below.
    ///
    /// This handles sending and receiving data on the socket. When using
    /// `Lines`, we can work at the line level instead of having to manage the
    /// raw byte operations.
    pub lines: Framed<TcpStream, LinesCodec>,

    /// Receive half of the message channel.
    ///
    /// This is used to receive messages from peers. When a message is received
    /// off of this `Rx`, it will be written to the socket.
    pub rx: Rx,
}

// The user might expect to be able to use Default as the type can be constructed without arguments.
// https://rust-lang.github.io/rust-clippy/master/index.html#new_without_default
impl Default for Shared {
    fn default() -> Self {
        Self::new()
    }
}

impl Shared {
    /// Create a new, empty, instance of `Shared`.
    pub fn new() -> Self {
        Shared {
            peers: HashMap::new(),
        }
    }

    /// Send a `LineCodec` encoded message to every peer, except
    /// for the sender.
    pub async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
        for peer in self.peers.iter_mut() {
            if *peer.0 != sender {
                let _ = peer.1.tx.send(message.into());
            }
        }
    }

    pub fn send_to_sender(&mut self, sender: SocketAddr, message: &str) {
        let _ = self.peers[&sender].tx.send(message.into());
    }

    pub fn get_everyone(&mut self, sender: SocketAddr) -> Vec<String> {
        let mut everyone = vec![];
        for peer in self.peers.iter() {
            if *peer.0 != sender {
                everyone.push(peer.1.username.clone());
            }
        }
        everyone
    }
    
}

impl Peer {
    /// Create a new instance of `Peer`.
    pub async fn new(
        state: Arc<Mutex<Shared>>,
        lines: Framed<TcpStream, LinesCodec>,
        username: String,
    ) -> io::Result<Peer> {
        // Get the client socket address
        let addr = lines.get_ref().peer_addr()?;

        // Create a channel for this peer
        let (tx, rx) = mpsc::unbounded_channel();

        // Add an entry for this `Peer` in the shared state map.
        state
            .lock()
            .await
            .peers
            .insert(addr, UserDetails { username, tx });

        Ok(Peer { lines, rx })
    }
}

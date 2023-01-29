#![warn(rust_2018_idioms)]

use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;
use std::net::SocketAddr;
use std::{env, io};
use tokio::net::UdpSocket;

use env_logger::Env;
use log::info;

struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
    received: Option<(usize, SocketAddr)>,
    db: HashMap<String, String>,
}

/// There are only two types of requests: insert and retrieve.
/// Insert allows a client to insert a value for a key,
/// and retrieve allows a client to retrieve the value for a key.
// enum MessageType {
//     Insert,
//     Retrieve,
// }

/// This function returns the key & value pair
/// if this is an INSERT op, or None if this is a RETRIEVE op
/// https://doc.rust-lang.org/std/primitive.str.html#method.split_once
fn get_kv_pair(msg: &[u8]) -> Option<(String, String)> {
    String::from_utf8_lossy(msg)
        .split_once('=')
        .map(|(key, value)| -> (String, String) { (key.to_string(), value.to_string()) })
}

impl Server {
    async fn run(self) -> Result<(), io::Error> {
        let Server {
            socket,
            mut buf,
            mut received,
            mut db,
        } = self;

        loop {
            // First we check to see if there's a message we need to process.
            if let Some((size, peer)) = received {
                let msg = &buf[..size];
                info!("Received a msg from {}", peer);

                // If this comes back with something, that means this was an Insert,
                // so we just add it to hashmap
                if let Some((k, v)) = get_kv_pair(msg) {
                    info!("Insert message type detected, adding.");
                    db.insert(k, v);
                } else {
                    info!("Retreive message type detected, replying.");
                }
            }

            // If we're here then `received` is `None`, so we look for the next message to process.
            received = Some(socket.recv_from(&mut buf).await?);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup the logging framework
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:8080".to_string());

    let socket = UdpSocket::bind(&addr).await?;
    info!("Listening on: {}", socket.local_addr()?);

    let server = Server {
        socket,
        buf: vec![0; 1024],
        received: None,
        db: HashMap::new(),
    };

    // This starts the server task.
    server.run().await?;

    Ok(())
}

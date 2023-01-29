#![warn(rust_2018_idioms)]

use std::collections::HashMap;
use std::error::Error;
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
                match get_kv_pair(msg) {
                    Some((k, v)) => {
                        info!("Insert message type detected, adding {}={}", k, v);
                        db.insert(k, v);
                    }
                    None => {
                        // OK, it's a Retrieve type, let's convert the message to a String
                        // and pull the value from the HashMap
                        let key_as_string = String::from_utf8(msg.to_vec()).unwrap();

                        // if this k,v exists, we send it back. If not, we go silent and ignore.
                        if let Some(reply) = db.get(&key_as_string){
                            info!("Retrieve message type detected, replying with {}", reply);
                            let _amt = socket.send_to(reply.as_bytes(), &peer).await?;    
                        }
                    }
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

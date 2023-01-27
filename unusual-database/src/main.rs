#![warn(rust_2018_idioms)]

use anyhow::{Context, Ok, Result};

use std::net::SocketAddr;
use std::{env, io};
use tokio::net::UdpSocket;


use log::{error, info};

/// There are only two types of requests: insert and retrieve.
/// Insert allows a client to insert a value for a key,
/// and retrieve allows a client to retrieve the value for a key.
enum MessageType {
    Insert,
    Retrieve,
}

fn get_message_type(msg: &[u8]) -> MessageType {
    for ch in msg.iter() {
        if char::from(*ch) == '=' {
            info!("Received an insert message");
            return MessageType::Insert
        }
    }
    info!("Received an retrieve message");
    
    MessageType::Retrieve
}

struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
    to_send: Option<(usize, SocketAddr)>,
}

impl Server {
    async fn run(self) -> Result<(), io::Error> {
        let Server {
            socket,
            mut buf,
            mut to_send,
        } = self;

        loop {
            // First we check to see if there's a message we need to echo back.
            // If so then we try to send it back to the original source, waiting
            // until it's writable and we're able to do so.
            if let Some((size, peer)) = to_send {
                // let amt = socket.send_to(&buf[..size], &peer).await?;
                
                info!("Received {:?} from {}", buf, peer);
                let message_type = get_message_type(&buf);

                // println!("Echoed {}/{} bytes to {}", amt, size, peer);
            }

            // If we're here then `to_send` is `None`, so we take a look for the
            // next message we're going to echo back.
            to_send = Some(socket.recv_from(&mut buf).await?);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let socket = UdpSocket::bind(&addr).await?;
    println!("Listening on: {}", socket.local_addr()?);

    let server = Server {
        socket,
        buf: vec![0; 1024],
        to_send: None,
    };

    // This starts the server task.
    server.run().await?;

    Ok(())
}

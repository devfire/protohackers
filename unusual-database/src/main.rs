#![warn(rust_2018_idioms)]

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
}

impl Server {
    async fn run(self) -> Result<(), io::Error> {
        let Server {
            socket,
            mut buf,
            mut received,
        } = self;

        loop {
            // First we check to see if there's a message we need to process.
            // If so then we try to send it back to the original source, waiting
            // until it's writable and we're able to do so.
            if let Some((size, peer)) = received {
                // let amt = socket.send_to(&buf[..size], &peer).await?;

                info!("Received {} bytes from {}", size, peer);
            }

            /*
            If we're here then `received` is `None`, so we take a look for the
            next message we're going to process.
            */
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
    };

    // This starts the server task.
    server.run().await?;

    Ok(())
}

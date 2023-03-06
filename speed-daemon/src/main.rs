// use std::sync::Arc;
use rusqlite::{params, Result};
use speed_daemon::{codec::MessageCodec, message::InboundMessageType};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};
use tokio_rusqlite::Connection;

use env_logger::Env;
use log::{error, info};
use tokio_util::codec::FramedRead;

use futures::{SinkExt, Stream, StreamExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup the logging framework
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    info!("Starting the speed daemon server.");

    let conn = Connection::open_in_memory().await?;

    // Create the shared state. This is how all the peers communicate.
    //
    let heartbeat = conn
        .call(|conn| {
            conn.execute(
                "CREATE TABLE heartbeat (
            id INTEGER PRIMARY KEY,
            ip TEXT NOT NULL,
            interval INTEGER)",
                [],
            )
            .unwrap();
        })
        .await;

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);

    // Bind a TCP listener to the socket address.
    //
    // Note that this is the Tokio TcpListener, which is fully async.
    let listener = TcpListener::bind(&addr).await?;

    info!("Server running on {}", addr);

    loop {
        // Asynchronously wait for an inbound TcpStream.
        let (stream, addr) = listener.accept().await?;

        // Clone a handle to the `Shared` state for the new connection.
        let conn = conn.clone();
            
        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            info!("Accepted connection from {}", addr);
            
            if let Err(e) = process(stream, addr, &conn).await {
                info!("an error occurred; error = {:?}", e);
            }
        });
    }
}

async fn process(stream: TcpStream, addr: SocketAddr, connection: &Connection) -> anyhow::Result<()> {
    info!("Processing stream from {}", addr);
    let (client_reader, mut client_writer) = stream.into_split();

    let mut client_reader = FramedRead::new(client_reader, MessageCodec::new());

    while let Some(message) = client_reader.next().await {
        info!("From {}: {:?}", addr, message);

        match message {
            Ok(InboundMessageType::Plate { plate, timestamp }) => handle_plate(plate, timestamp),

            Ok(InboundMessageType::Ticket {
                plate,
                road,
                mile1,
                timestamp1,
                mile2,
                timestamp2,
                speed,
            }) => handle_ticket(InboundMessageType::Ticket {
                plate,
                road,
                mile1,
                timestamp1,
                mile2,
                timestamp2,
                speed,
            }),

            Ok(InboundMessageType::WantHeartbeat { interval }) => {
                handle_want_hearbeat(addr, interval)
            }

            Ok(InboundMessageType::IAmCamera { road, mile, limit }) => {
                handle_i_am_camera(InboundMessageType::IAmCamera { road, mile, limit })
            }

            Ok(InboundMessageType::IAmDispatcher { numroads, roads }) => {
                handle_i_am_dispatcher(InboundMessageType::IAmDispatcher { numroads, roads })
            }
            Err(_) => error!("Unknown message detected"),
        }
    }
    Ok(())
}

fn handle_plate(plate: String, timestamp: u32) {
    todo!()
}

fn handle_ticket(message: InboundMessageType) {
    todo!()
}

fn handle_want_hearbeat(client_address: SocketAddr, interval: u32) {
    info!(
        "Client {} requested a heartbeat every {} deciseconds.",
        client_address, interval
    )
}

fn handle_i_am_camera(message: InboundMessageType) {
    todo!()
}

fn handle_i_am_dispatcher(message: InboundMessageType) {
    todo!()
}

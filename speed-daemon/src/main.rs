// use std::sync::Arc;
use speed_daemon::{codec::MessageCodec, errors::SpeedDaemonError, message::InboundMessageType};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::{TcpListener, TcpStream};

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

    // Create the shared state. This is how all the peers communicate.
    //
    // The server task will hold a handle to this. For every new client, the
    // `state` handle is cloned and passed into the task that processes the
    // client connection.
    // let state = Arc::new(tokio::sync::Mutex::new(Shared::new()));

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);

    // Bind a TCP listener to the socket address.
    //
    // Note that this is the Tokio TcpListener, which is fully async.
    let listener = TcpListener::bind(&addr).await?;

    info!("server running on {}", addr);

    loop {
        // Asynchronously wait for an inbound TcpStream.
        let (stream, addr) = listener.accept().await?;

        // Clone a handle to the `Shared` state for the new connection.
        // let state = Arc::clone(&state);

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            info!("accepted connection from {}", addr);
            // if let Err(e) = process(state, stream, addr).await {
            if let Err(e) = process(stream, addr).await {
                info!("an error occurred; error = {:?}", e);
            }
        });
    }
}

async fn process(stream: TcpStream, addr: SocketAddr) -> anyhow::Result<()> {
    info!("Processing stream from {}", addr);
    let (client_reader, mut client_writer) = stream.into_split();

    let mut client_reader = FramedRead::new(client_reader, MessageCodec::new());

    while let Some(message) = client_reader.next().await {
        info!("From {}: {:?}", addr, message);

        match message {
            Ok(InboundMessageType::Plate { plate, timestamp }) => {
                handle_plate(InboundMessageType::Plate { plate, timestamp })
            }

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
                handle_want_hearbeat(InboundMessageType::WantHeartbeat { interval })
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

fn handle_plate(message: InboundMessageType::Plate) {
    todo!()
}

fn handle_ticket(message: InboundMessageType::Ticket) {
    todo!()
}

fn handle_want_hearbeat(message: InboundMessageType::WantHeartbeat) {
    todo!()
}

fn handle_i_am_camera (message: InboundMessageType::IAmCamera) {
    todo!()
}

fn handle_i_am_dispatcher(message: InboundMessageType::IAmDispatcher) {
    todo!()
}
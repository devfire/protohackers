// use std::sync::Arc;
use speed_daemon::{
    codec::MessageCodec,
    message::{InboundMessageType, OutboundMessageType},
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::{TcpListener, TcpStream};

use env_logger::Env;
use log::{error, info};
use tokio_util::codec::Framed;

use futures::TryStreamExt;

use tokio::time::{sleep, Duration};

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

    info!("Server running on {}", addr);

    loop {
        // Asynchronously wait for an inbound TcpStream.
        let (stream, addr) = listener.accept().await?;

        // Clone a handle to the `Shared` state for the new connection.
        // let state = Arc::clone(&state);

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            info!("Accepted connection from {}", addr);
            // if let Err(e) = process(state, stream, addr).await {
            if let Err(e) = process(stream, addr).await {
                error!("an error occurred; error = {:?}", e);
            }
        });
    }
}

async fn process(stream: TcpStream, addr: SocketAddr) -> anyhow::Result<()> {
    info!("Processing stream from {}", addr);
    // let (client_reader, client_writer) = stream.into_split();

    // let mut client_reader = FramedRead::new(client_reader, MessageCodec::new());
    // let mut client_writer = FramedWrite::new(client_writer, MessageCodec::new());

    let mut framed: Framed<TcpStream, MessageCodec> = Framed::new(stream, MessageCodec {});
    while let Some(message) = framed.try_next().await? {
        info!("From {}: {:?}", addr, message);

        match message {
            InboundMessageType::Plate { plate, timestamp } => handle_plate(plate, timestamp),

            InboundMessageType::Ticket {
                plate,
                road,
                mile1,
                timestamp1,
                mile2,
                timestamp2,
                speed,
            } => handle_ticket(InboundMessageType::Ticket {
                plate,
                road,
                mile1,
                timestamp1,
                mile2,
                timestamp2,
                speed,
            }),

            InboundMessageType::WantHeartbeat { interval } => {
                info!(
                    "Client {} requested heartbeat every {} deciseconds.",
                    addr, interval
                );

                // Spawn our handler to be run asynchronously.
                tokio::spawn(async move {
                    loop {
                        sleep(Duration::from_secs(interval as u64)).await;
                    }
                });
            }

            InboundMessageType::IAmCamera { road, mile, limit } => {
                handle_i_am_camera(InboundMessageType::IAmCamera { road, mile, limit })
            }

            InboundMessageType::IAmDispatcher { numroads, roads } => {
                handle_i_am_dispatcher(InboundMessageType::IAmDispatcher { numroads, roads })
            }
        }
    } // end of while
    Ok(())
}

#[allow(unused)]
fn handle_plate(plate: String, timestamp: u32) {
    todo!()
}
#[allow(unused)]
fn handle_ticket(message: InboundMessageType) {
    todo!()
}
#[allow(unused)]
fn handle_want_hearbeat(interval: u32) {
    todo!()
}
#[allow(unused)]
fn handle_i_am_camera(message: InboundMessageType) {
    todo!()
}
#[allow(unused)]
fn handle_i_am_dispatcher(message: InboundMessageType) {
    todo!()
}
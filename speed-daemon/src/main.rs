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

use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup the logging framework
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    info!("Starting the speed daemon server.");

    // Create the shared state. This is how all the peers communicate.

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

    // A unified [`Stream`] and [`Sink`] interface to an underlying I/O object, using
    // the `Encoder` and `Decoder` traits to encode and decode frames.
    // NOTE: split is not necessary here since Framed does both read & write.
    let mut framed: Framed<TcpStream, MessageCodec> = Framed::new(stream, MessageCodec {});

    // Create a new channel with a capacity of at most 32.
    // The mpsc channel is used to send commands to the task managing the client connection.
    // The multi-producer capability allows messages to be sent from many tasks.
    // Creating the channel returns two values, a sender and a receiver.
    // The two handles are used separately. They may be moved to different tasks.
    let (tx, mut rx) = mpsc::channel::<OutboundMessageType>(32);

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
                let heartbeat_tx = tx.clone();
                // Spawn our handler to be run asynchronously.
                tokio::spawn(async move {
                    handle_want_hearbeat(heartbeat_tx, interval).await;
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
async fn handle_want_hearbeat(tx: tokio::sync::mpsc::Sender<OutboundMessageType>, interval: u32) {
    loop {
        sleep(Duration::from_secs(interval as u64)).await;
    }
}
#[allow(unused)]
fn handle_i_am_camera(message: InboundMessageType) {
    todo!()
}
#[allow(unused)]
fn handle_i_am_dispatcher(message: InboundMessageType) {
    todo!()
}

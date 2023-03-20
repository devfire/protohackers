use speed_daemon::{
    codec::MessageCodec,
    errors::SpeedDaemonError,
    message::{InboundMessageType, OutboundMessageType},
    state::Db,
};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

use env_logger::Env;
use log::{error, info, warn};
use tokio_util::codec::{FramedRead, FramedWrite};

use futures::sink::SinkExt;
use futures::StreamExt;

mod handlers;

use handlers::{handle_i_am_camera, handle_i_am_dispatcher, handle_want_hearbeat};

use crate::handlers::{handle_error, handle_plate};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup the logging framework
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    info!("Starting the speed daemon server.");

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);

    let shared_db = Db::new();

    // Bind a TCP listener to the socket address.
    //
    // Note that this is the Tokio TcpListener, which is fully async.
    let listener = TcpListener::bind(&addr).await?;

    let (ticket_tx, mut ticket_rx) = mpsc::channel::<OutboundMessageType>(32);
    // NOTE: These are cheap clones, pointers only.
    let ticket_tx_queue = ticket_tx.clone();
    let shared_ticket_db = shared_db.clone();

    // Spawn off a ticket manager loop.
    tokio::spawn(async move {
        // Start receiving messages from the channel by calling the recv method of the Receiver endpoint.
        // This method blocks until a message is received.
        while let Some(ticket) = ticket_rx.recv().await {
            info!("Ticket manager received {:?}", ticket);

            if let OutboundMessageType::Ticket {
                plate: _,
                road,
                mile1: _,
                timestamp1: _,
                mile2: _,
                timestamp2: _,
                speed: _,
            } = ticket
            {
                if let Some(dispatcher_tx) = shared_ticket_db.get_ticket_dispatcher(road) {
                    send_ticket_to_dispatcher(ticket, dispatcher_tx);
                } else {
                    warn!("No dispatcher found, sending the ticket back");
                    ticket_tx_queue
                        .send(ticket)
                        .await
                        .expect("Unable to send ticket");
                }
            }
        }
    });

    info!("Server running on {}", addr);

    loop {
        // Asynchronously wait for an inbound TcpStream.
        let (stream, addr) = listener.accept().await?;

        // Clone the handle to the shared state.
        let shared_db_main = shared_db.clone();
        let ticket_tx_process = ticket_tx.clone();
        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            info!("Accepted connection from {}", addr);
            if let Err(e) = process(stream, addr, ticket_tx_process, shared_db_main).await {
                error!("Error: {:?}", e);
            }
        });
    }
}

async fn process(
    stream: TcpStream,
    addr: SocketAddr,
    ticket_tx: mpsc::Sender<OutboundMessageType>,
    shared_db: Db,
) -> anyhow::Result<()> {
    info!("Processing stream from {}", addr);
    let (client_reader, client_writer) = stream.into_split();

    let mut client_reader = FramedRead::new(client_reader, MessageCodec::new());
    let mut client_writer = FramedWrite::new(client_writer, MessageCodec::new());

    // The mpsc channel is used to send commands to the task managing the client connection.
    // The multi-producer capability allows messages to be sent from many tasks.
    // Creating the channel returns two values, a sender and a receiver.
    // The two handles are used separately. They may be moved to different tasks.
    //
    // NOTE: The channel is created with a capacity of 32.
    // If messages are sent faster than they are received, the channel will store them.
    // Once the 32 messages are stored in the channel,
    // calling send(...).await will go to sleep until a message has been removed by the receiver.
    let (tx, mut rx) = mpsc::channel::<OutboundMessageType>(32);

    // Spawn off a writer manager loop.
    // In order to send a message back to the clients, all threads must use mpsc channel to publish data.
    // The manager will then proxy the data and send it on behalf of threads.
    let manager = tokio::spawn(async move {
        // Start receiving messages from the channel by calling the recv method of the Receiver endpoint.
        // This method blocks until a message is received.
        while let Some(msg) = rx.recv().await {
            info!("Writer manager: sending {:?} to {}", msg, addr);

            if let Err(e) = client_writer.send(msg).await {
                error!("Client {} disconnected: {}", addr, e);
                return Err(SpeedDaemonError::DisconnectedClient);
            }
        }
        Ok(())
    });

    while let Some(message) = client_reader.next().await {
        info!("From {}: {:?}", addr, message);

        match message {
            Ok(InboundMessageType::Plate { plate, timestamp }) => {
                handle_plate(
                    &addr,
                    plate,
                    timestamp,
                    ticket_tx.clone(),
                    shared_db.clone(),
                )
                .await?
            }

            Ok(InboundMessageType::WantHeartbeat { interval }) => {
                info!(
                    "Client {} requested a heartbeat every {} deciseconds.",
                    addr, interval
                );
                // if interal is 0 then no heartbeat
                if interval == 0 {
                    info!("Interval is 0, no heartbeat.")
                } else {
                    let tx_heartbeat = tx.clone();
                    handle_want_hearbeat(interval, tx_heartbeat)?;
                }
            }

            Ok(InboundMessageType::IAmCamera { road, mile, limit }) => {
                let new_camera = InboundMessageType::IAmCamera { road, mile, limit };
                handle_i_am_camera(&addr, new_camera, shared_db.clone())?;
            }

            Ok(InboundMessageType::IAmDispatcher { roads }) => {
                info!("Dispatcher detected at address {}", addr);

                handle_i_am_dispatcher(roads, &addr, &tx, shared_db.clone()).await?;
            }
            Err(_) => {
                let err_message = String::from("Unknown message detected");
                error!("{}", err_message);
                let tx_error = tx.clone();
                handle_error(err_message, tx_error)?;
            }
        }
    }

    if let Err(e) = manager.await? {
        error!("Error from the tx manager: {}", e)
    }
    Ok(())
}

fn send_ticket_to_dispatcher(ticket: OutboundMessageType, tx: mpsc::Sender<OutboundMessageType>) {
    info!("Sending {:?} to writer manager ", ticket);
    tokio::spawn(async move {
        tx.send(ticket).await.expect("Unable to send ticket");
    });
}

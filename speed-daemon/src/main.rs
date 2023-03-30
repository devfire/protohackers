use speed_daemon::{
    codec::MessageCodec,
    errors::SpeedDaemonError,
    message::{InboundMessageType, OutboundMessageType},
    state::Db,
    types::PlateRoadStruct,
};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

use env_logger::Env;
use log::{error, info};
use tokio_util::codec::{FramedRead, FramedWrite};

use futures::sink::SinkExt;
use futures::StreamExt;

mod handlers;

use handlers::{handle_i_am_camera, handle_i_am_dispatcher, handle_want_hearbeat};

use crate::handlers::{handle_error, handle_plate};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    // console_subscriber::init();
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

    info!("Server running on {}", addr);

    loop {
        // Asynchronously wait for an inbound TcpStream.
        let (stream, addr) = listener.accept().await?;
        let shared_db_main = shared_db.clone();

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            // info!("Accepted connection from {}", addr);
            if let Err(e) = process(stream, addr, shared_db_main).await {
                error!("Error: {:?}", e);
            }
        });
    }
}

async fn process(
    stream: TcpStream,
    addr: SocketAddr,
    shared_db: Db,
) -> anyhow::Result<(), SpeedDaemonError> {
    // info!("Processing stream from {}", addr);
    let (client_reader, client_writer) = stream.into_split();

    let mut client_reader = FramedRead::new(client_reader, MessageCodec::new());
    let mut client_writer = FramedWrite::new(client_writer, MessageCodec::new());

    // The mpsc channel is used to send commands to the task managing the client connection.
    // The multi-producer capability allows messages to be sent from many tasks.
    // Creating the channel returns two values, a sender and a receiver.
    // The two handles are used separately. They may be moved to different tasks.
    //
    // NOTE: The channel is created with a certain capacity.
    // If messages are sent faster than they are received, the channel will store them.
    // Once the N messages are stored in the channel,
    // calling send(...).await will go to sleep until a message has been removed by the receiver.
    let (tx, mut rx) = mpsc::channel::<OutboundMessageType>(819600);

    // Spawn off a writer manager loop.
    // In order to send a message back to the clients, all threads must use mpsc channel to publish data.
    // The manager will then proxy the data and send it on behalf of threads.
    let manager = tokio::spawn(async move {
        // Start receiving messages from the channel by calling the recv method of the Receiver endpoint.
        // This method blocks until a message is received.
        while let Some(msg) = rx.recv().await {
            // info!("Writer manager: sending {:?} to {}", msg, addr);

            client_writer.send(msg).await?
        }
        Ok::<(), SpeedDaemonError>(())
    });

    let (ticket_tx, mut ticket_rx) = mpsc::channel::<OutboundMessageType>(8192000);

    // Clone the handle to the shared state.
    let shared_db_main = shared_db.clone();

    // NOTE: These are cheap clones, pointers only.
    let ticket_tx_queue = ticket_tx.clone();

    // Spawn off a ticket manager loop.
    let ticket_manager = tokio::spawn(async move {
        // Start receiving messages from the channel by calling the recv method of the Receiver endpoint.
        // This method blocks until a message is received.
        while let Some(ticket) = ticket_rx.recv().await {
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
                if let Some(dispatcher_tx) = shared_db_main.get_ticket_dispatcher(road).await {
                    // info!("Ticket manager received {:?}", ticket);
                    send_ticket_to_dispatcher(ticket, dispatcher_tx);
                } else {
                    // warn!("No dispatcher found, sending the ticket back");
                    ticket_tx_queue
                        .send(ticket)
                        .await
                        .expect("Unable to send ticket");
                }
            }
        }
        Ok::<(), SpeedDaemonError>(())
    });

    let (plate_tx, mut plate_rx) = mpsc::channel::<PlateRoadStruct>(8192000);
    // this channel is for plate handler -> plate ticket checker
    let plate_tx_queue = ticket_tx.clone();

    let shared_db_plate = shared_db.clone();

    // This receives messages from the plate_handler. Checks each one to see if we need to generate at ticket.
    // Sends a ticket to the ticket dispatcher if yes.
    let plate_manager = tokio::spawn(async move {
        while let Some(new_plate_road) = plate_rx.recv().await {
            if let Some(tickets) = shared_db_plate.get_ticket_for_plate(&new_plate_road).await {
                info!(
                    "Plate manager forwarding tickets {:?} to ticket manager",
                    tickets
                );

                for ticket in tickets.iter() {
                    // Send the ticket to the ticket dispatcher
                    plate_tx_queue
                        .send(ticket.to_owned())
                        .await
                        .expect("Unable to send ticket");
                }
            }
        }
        Ok::<(), SpeedDaemonError>(())
    });

    while let Some(message) = client_reader.next().await {
        // info!("From {}: {:?}", addr, message);

        match message {
            Ok(InboundMessageType::Plate { plate, timestamp }) => {
                handle_plate(&addr, plate, timestamp, plate_tx.clone(), shared_db.clone())
                    .await
                    .expect("unable to handle plates");
            }

            Ok(InboundMessageType::WantHeartbeat { interval }) => {
                // if interal is 0 then no heartbeat
                if interval == 0 {
                    // info!("Interval is 0, no heartbeat.")
                } else {
                    let tx_heartbeat = tx.clone();
                    handle_want_hearbeat(interval, tx_heartbeat)
                        .await
                        .expect("Unable to heartbeat");
                }
            }

            Ok(InboundMessageType::IAmCamera { road, mile, limit }) => {
                let new_camera = InboundMessageType::IAmCamera { road, mile, limit };
                handle_i_am_camera(&addr, new_camera, shared_db.clone())
                    .await
                    .expect("Unable to handle camera");
            }

            Ok(InboundMessageType::IAmDispatcher { roads }) => {
                // info!("Dispatcher detected at address {}", addr);
                handle_i_am_dispatcher(roads, &addr, &tx, shared_db.clone())
                    .await
                    .expect("Unable to handle dispatcher");
            }
            Err(_) => {
                let err_message = String::from("Unknown message detected");
                // error!("{}", err_message);
                let tx_error = tx.clone();
                handle_error(err_message, tx_error).expect("Unable to handle error");
            }
        }
    }

    if let Err(e) = manager.await.expect("Unable to await msg manager") {
        error!("Error from the tx manager: {}", e)
    }

    if let Err(e) = ticket_manager
        .await
        .expect("Unable to await ticket manager")
    {
        error!("Error from the tx manager: {}", e)
    }

    if let Err(e) = plate_manager.await.expect("Unable to await msg manager") {
        error!("Error from the tx manager: {}", e)
    }

    Ok(())
}

fn send_ticket_to_dispatcher(ticket: OutboundMessageType, tx: mpsc::Sender<OutboundMessageType>) {
    // info!("Sending {:?} to writer manager ", ticket);
    tokio::spawn(async move {
        tx.send(ticket).await.expect("Unable to send ticket");
    });
}

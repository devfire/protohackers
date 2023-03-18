use speed_daemon::{
    codec::MessageCodec,
    message::{InboundMessageType, OutboundMessageType},
    state::SharedState,
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

use std::sync::{Arc, Mutex};

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

    // let db = Arc::new(Mutex::new(HashMap::new()));
    // let ticket_dispatcher_db: Arc<
    //     Mutex<HashMap<u16, Vec<tokio::sync::mpsc::Sender<OutboundMessageType>>>>,
    // > = Arc::new(Mutex::new(HashMap::new()));

    // let dispatchers: TicketDispatcherDb;
    // let current_camera: InboundMessageType;
    // let plates_cameras: PlateCameraDb;

    let shared_db = Arc::new(Mutex::new(SharedState::new()));

    // let shared_db = shared_db.lock()?;

    // let mut hash_of_hashes: HashMap<InboundMessageType, HashMap<InboundMessageType, u32>> = HashMap::new();

    // Bind a TCP listener to the socket address.
    //
    // Note that this is the Tokio TcpListener, which is fully async.
    let listener = TcpListener::bind(&addr).await?;

    info!("Server running on {}", addr);

    loop {
        // Asynchronously wait for an inbound TcpStream.
        let (stream, addr) = listener.accept().await?;

        // Clone the handle to the shared state.
        let shared_db = Arc::clone(&shared_db);

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            info!("Accepted connection from {}", addr);
            if let Err(e) = process(stream, addr, shared_db).await {
                info!("an error occurred; error = {:?}", e);
            }
        });
    }
}

async fn process(
    stream: TcpStream,
    addr: SocketAddr,
    shared_db: Arc<Mutex<SharedState>>,
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
            info!("Sending {:?} to {}", msg, addr);

            if let Err(e) = client_writer.send(msg).await {
                error!("Client {} disconnected: {}", addr, e);
                client_writer
                    .close()
                    .await
                    .expect("Unable to close channel.");
            }
        }
    });

    // Clone the handle to the shared state.
    // let shared_db = shared_db.clone();
    tokio::spawn(async move {
        check_ticket_queue(shared_db).await;
    });

    while let Some(message) = client_reader.next().await {
        info!("From {}: {:?}", addr, message);

        match message {
            Ok(InboundMessageType::Plate { plate, timestamp }) => {
                handle_plate(&addr, plate, timestamp, shared_db.clone()).await?
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
                    handle_want_hearbeat(interval, tx_heartbeat);
                }
            }

            Ok(InboundMessageType::IAmCamera { road, mile, limit }) => {
                let new_camera = InboundMessageType::IAmCamera { road, mile, limit };
                handle_i_am_camera(&addr, new_camera, shared_db.clone())?;
            }

            Ok(InboundMessageType::IAmDispatcher { roads }) => {
                info!("Dispatcher detected at address {}", addr);
                let shared_db = shared_db.clone();
                handle_i_am_dispatcher(roads, &addr, &tx, shared_db).await?;
            }
            Err(_) => {
                let err_message = String::from("Unknown message detected");
                error!("{}", err_message);
                let tx_error = tx.clone();
                handle_error(err_message, tx_error)?;
            }
        }
    }

    manager.await?;
    Ok(())
}

async fn check_ticket_queue(shared_db: Arc<Mutex<SharedState>>) {
    let mut shared_db = shared_db
        .lock()
        .expect("Unable to lock shared db in queue manager");
    // Keep checking for new tickets in the queue
    while let Some(new_ticket) = shared_db.get_ticket() {
        info!("Found a ticket {:?} to", new_ticket);

        // get the Road from the ticket
        if let OutboundMessageType::Ticket {
            ref plate,
            road,
            mile1,
            timestamp1,
            mile2,
            timestamp2,
            speed,
        } = new_ticket
        {
            // see if there's a tx for that road
            if let Some(tx) = shared_db.get_ticket_dispatcher(road) {
                info!("Found a dispatcher for the road {}", road);

                send_ticket_to_dispatcher(new_ticket, tx.clone());
            }
        }
    }
}

fn send_ticket_to_dispatcher(ticket: OutboundMessageType, tx: mpsc::Sender<OutboundMessageType>) {
    tokio::spawn(async move {
        tx.send(ticket).await.expect("Unable to send ticket");
    });
}

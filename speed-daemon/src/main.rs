use speed_daemon::{
    codec::MessageCodec,
    message::{InboundMessageType, OutboundMessageType},
    state::SharedState,
    types::{Mile, Plate, Road, Timestamp},
};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
    time::{sleep, Duration},
};

use env_logger::Env;
use log::{error, info};
use tokio_util::codec::{FramedRead, FramedWrite};

use futures::sink::SinkExt;
use futures::StreamExt;

use std::sync::{Arc, Mutex};

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
        let shared_db = shared_db.clone();
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
                handle_i_am_dispatcher( roads, &addr, &tx, shared_db.clone()).await?;
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

fn handle_error(
    error_message: String,
    tx: mpsc::Sender<OutboundMessageType>,
) -> anyhow::Result<()> {
    tokio::spawn(async move {
        tx.send(OutboundMessageType::Error(error_message))
            .await
            .expect("Unable to send error message");
    });
    Ok(())
}

async fn handle_plate(
    client_addr: &SocketAddr,
    new_plate: Plate,
    new_timestamp: Timestamp,
    shared_db: Arc<Mutex<SharedState>>,
) -> anyhow::Result<()> {
    let mut shared_db = shared_db.lock().expect("Unable to lock shared db");

    // info!("Received plate: {:?}", new_plate);

    // Get the current road speed limit
    let mut speed_limit: u16 = 0;
    let mut observed_mile_marker: Mile = 0;
    let mut current_road: Road = 0;

    // At this point, current_camera contains the InboundMessageType::IAmCamera enum with the current tokio task values
    // let new_camera = shared_db.current_camera.clone();
    let current_camera = shared_db.get_current_camera(client_addr);

    // Get the details of the camera that obseved this plate.
    // NOTE: this came from handle_i_am_camera
    if let InboundMessageType::IAmCamera { road, mile, limit } = *current_camera {
        current_road = road;
        observed_mile_marker = mile;
        speed_limit = limit;
    }

    info!(
        "Speed limit is: {} mile marker: {}",
        current_road, observed_mile_marker
    );

    let mut mile1: u16 = 0;
    let mut mile2: u16 = 0;
    let mut timestamp1: u32 = 0;
    let mut timestamp2: u32 = 0;
    // Check if this plate has been observed before
    if let Some(previously_seen_camera) = shared_db.plates_cameras.get(&new_plate) {
        let time_traveled: u32;
        let mut distance_traveled: u16 = 0;
        // Messages may arrive out of order, so we need to figure out what to subtract from what.
        // NOTE: previously_seen_camera is a (timestamp, InboundMessageType::IAmCamera) tuple,
        // so 0th entry refers to the timestamp.
        if new_timestamp > previously_seen_camera.0 {
            time_traveled = new_timestamp - previously_seen_camera.0;
            if let InboundMessageType::IAmCamera {
                road: _,
                mile,
                limit: _,
            } = previously_seen_camera.1
            {
                distance_traveled = observed_mile_marker - mile;
                mile1 = observed_mile_marker;
                mile2 = mile;
                timestamp1 = new_timestamp;
                timestamp2 = previously_seen_camera.0;
            }
        } else {
            time_traveled = previously_seen_camera.0 - new_timestamp;
            if let InboundMessageType::IAmCamera {
                road: _,
                mile,
                limit: _,
            } = previously_seen_camera.1
            {
                distance_traveled = mile - observed_mile_marker;
                mile1 = mile;
                mile2 = observed_mile_marker;
                timestamp1 = previously_seen_camera.0;
                timestamp2 = new_timestamp;
            }
        }

        let observed_speed: f64 = distance_traveled as f64 / time_traveled as f64 * 3600.0;
        info!(
            "Plate: {} seen by camera: {:?} distance traveled: {} in time: {} speed: {}",
            new_plate, previously_seen_camera, distance_traveled, time_traveled, observed_speed
        );

        // check if the car exceeded the speed limit
        if observed_speed > speed_limit as f64 {
            info!(
                "Plate {} exceeded the speed limit, issuing ticket",
                new_plate
            );
            let new_ticket = OutboundMessageType::Ticket {
                plate: new_plate,
                road: current_road,
                mile1,
                timestamp1,
                mile2,
                timestamp2,
                speed: observed_speed as u16,
            };

            // issue ticket
            // Get the relevant tx
            info!("Getting the relevant tx for road {} address {}", current_road, client_addr);
            let tx = shared_db.get_ticket_dispatcher(current_road, client_addr);

            let tx = tx.clone();
            issue_ticket(new_ticket, tx);
        }
    } else {
        info!(
            "First time seeing plate: {} observed by camera: {:?}",
            new_plate, shared_db.current_camera
        );
        // Add the newly observed camera to the shared db of plate -> camera hash
        // NOTE: subsequent inserts will override the value because the plate key is the same.
        // But that's OK since we only ever need the last two values.
        let current_camera = current_camera.clone();
        shared_db
            .plates_cameras
            .insert(new_plate, (new_timestamp, current_camera));
    }
    Ok(())
}

fn issue_ticket(ticket: OutboundMessageType, tx: mpsc::Sender<OutboundMessageType>) {
    tokio::spawn(async move {
        tx.send(ticket).await.expect("Unable to send ticket");
    });
}

fn handle_want_hearbeat(interval: u32, tx: mpsc::Sender<OutboundMessageType>) {
    tokio::spawn(async move {
        loop {
            tx.send(OutboundMessageType::Heartbeat)
                .await
                .expect("Unable to send heartbeat");
            let interval = interval / 10;
            sleep(Duration::from_secs(interval as u64)).await;
        }
    });
}

fn handle_i_am_camera(
    client_addr: &SocketAddr,
    new_camera: InboundMessageType,
    shared_db: Arc<Mutex<SharedState>>,
) -> anyhow::Result<()> {
    info!("Current camera: {:?}", new_camera);

    // Set the current tokio thread camera so we can look up its details later
    let mut shared_db = shared_db
        .lock()
        .expect("Unable to lock shared db in handle_i_am_camera");

    shared_db.add_camera(*client_addr, new_camera);

    Ok(())
}

async fn handle_i_am_dispatcher(
    roads: Vec<Road>,
    client_addr: &SocketAddr,
    tx: &mpsc::Sender<OutboundMessageType>,
    shared_db: Arc<Mutex<SharedState>>,
) -> anyhow::Result<()> {
    info!("Adding a dispatcher for roads {:?}", roads);
    // let mut shared_db = shared_db
      //   .lock()
        // .expect("Unable to lock shared db in handle_i_am_dispatcher");

    for road in roads.iter() {
        // for every road this dispatcher is responsible for, add the corresponding tx reference
        info!("Adding dispatcher {} for road {}", client_addr, road);
        // let tx = tx.clone();
        // shared_db.add_ticket_dispatcher(*road, *client_addr, tx)
    }
    Ok(())
}

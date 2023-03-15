use bytes::Bytes;
// use std::sync::Arc;
use speed_daemon::{
    codec::MessageCodec,
    message::{InboundMessageType, OutboundMessageType},
};
use std::{
    collections::HashMap,
    hash::Hash,
    // collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    os::unix::raw::time_t,
    sync::mpsc::Receiver,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
    time::{sleep, Duration},
};

use env_logger::Env;
use log::{error, info};
use tokio_util::codec::{FramedRead, FramedWrite};

use futures::sink::SinkExt;
use futures::{Stream, StreamExt};

use std::sync::{Arc, Mutex};

// Type alias CamelCase matches the db hash of hashes below
// type Db = Arc<Mutex<HashMap<u16, Bytes>>>;
// type Db = Arc<Mutex<HashMap<u16, Vec<InboundMessageType>>>>;
// type Db = Arc<Mutex<Vec<HashMap<InboundMessageType,InboundMessageType>>>>;

// A hash of Plate -> (timestamp, IAmCamera)
type Db = Arc<Mutex<HashMap<String, (u32, InboundMessageType)>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup the logging framework
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    info!("Starting the speed daemon server.");

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);

    let db = Arc::new(Mutex::new(HashMap::new()));

    // let mut hash_of_hashes: HashMap<InboundMessageType, HashMap<InboundMessageType, u32>> = HashMap::new();

    // Bind a TCP listener to the socket address.
    //
    // Note that this is the Tokio TcpListener, which is fully async.
    let listener = TcpListener::bind(&addr).await?;

    info!("Server running on {}", addr);

    loop {
        // Asynchronously wait for an inbound TcpStream.
        let (stream, addr) = listener.accept().await?;

        // Clone the handle to the hash map.
        let db = db.clone();

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            info!("Accepted connection from {}", addr);
            if let Err(e) = process(stream, addr, db).await {
                info!("an error occurred; error = {:?}", e);
            }
        });
    }
}

async fn process(stream: TcpStream, addr: SocketAddr, db: Db) -> anyhow::Result<()> {
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
        // Start receiving messages

        while let Some(msg) = rx.recv().await {
            info!("Sending {:?} to {}", msg, addr);
            client_writer
                .send(msg)
                .await
                .expect("Unable to send message");
        }
    });

    // This shared var holds the current thread camera. Init to 0 and overriden later.
    let current_camera = Arc::new(Mutex::new(InboundMessageType::IAmCamera {
        road: 0,
        mile: 0,
        limit: 0,
    }));

    while let Some(message) = client_reader.next().await {
        info!("From {}: {:?}", addr, message);

        match message {
            Ok(InboundMessageType::Plate { plate, timestamp }) => {
                handle_plate(plate, timestamp, db.clone(), current_camera.clone()).await?
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
                handle_i_am_camera(new_camera, &tx, current_camera.clone()).await?;
            }

            Ok(InboundMessageType::IAmDispatcher { numroads, roads }) => {
                handle_i_am_dispatcher(InboundMessageType::IAmDispatcher { numroads, roads })
            }
            Err(_) => {
                let err_message = String::from("Unknown message detected");
                error!("{}", err_message);
                let tx_error = tx.clone();
                handle_error(err_message, tx_error)?;
            }
        }
    }

    // .await the join handles to ensure the commands fully complete before the process exits.
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
    new_plate: String,
    new_timestamp: u32,
    db: Db,
    current_camera: Arc<Mutex<InboundMessageType>>,
) -> anyhow::Result<()> {
    let mut db = db.lock().expect("Unable to lock shared db");

    info!("Adding plate: {:?}", new_plate);

    let current_camera = current_camera
        .lock()
        .expect("Unable to lock the current road for editing");

    // db.push((new_plate, current_camera.clone()));

    // Get the current road speed limit
    let mut speed_limit: u16 = 0;
    let mut observed_mile_marker: u16 = 0;
    if let InboundMessageType::IAmCamera {
        road: _,
        mile,
        limit,
    } = current_camera.clone()
    {
        observed_mile_marker = mile;
        speed_limit = limit;
    }
    info!(
        "Speed limit is: {} mile marker: {}",
        speed_limit, observed_mile_marker
    );

    // Check if this plate has been observed before
    // let mut previously_seen_plate: (u16, InboundMessageType);
    // let mut previous_timestamp: InboundMessageType;

    // let mut previously_seen_plate: String;
    // let mut time_traveled: u32 = 0;
    let mut distance_traveled: u16 = 0;
    if let Some(previously_seen_camera) = db.get(&new_plate) {
        let time_traveled = previously_seen_camera.0 - new_timestamp;

        if let InboundMessageType::IAmCamera {
            road: _,
            mile,
            limit: _,
        } = previously_seen_camera.1
        {
            distance_traveled = observed_mile_marker - mile;
        }
        let observed_speed = distance_traveled as u32 / time_traveled / 3600;
        info!(
            "Plate: {} seen by camera: {:?} distance traveled: {} in time: {} speed: {}mph",
            new_plate, previously_seen_camera, distance_traveled, time_traveled, observed_speed
        );
    }

    Ok(())
}

#[allow(unused)]
fn handle_ticket(message: InboundMessageType) {
    todo!()
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

async fn handle_i_am_camera(
    new_camera: InboundMessageType,
    tx: &mpsc::Sender<OutboundMessageType>,
    my_camera: Arc<Mutex<InboundMessageType>>,
) -> anyhow::Result<()> {
    info!("Current camera: {:?}", new_camera);

    // Set the current tokio thread road so we can look up its details later
    let mut my_camera = my_camera
        .lock()
        .expect("Unable to lock the current road for editing");
    *my_camera = new_camera;

    Ok(())
}

#[allow(unused)]
fn handle_i_am_dispatcher(message: InboundMessageType) {
    todo!()
}

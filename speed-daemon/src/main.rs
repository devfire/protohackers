use bytes::Bytes;
// use std::sync::Arc;
use speed_daemon::{
    codec::MessageCodec,
    message::{InboundMessageType, OutboundMessageType},
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup the logging framework
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    info!("Starting the speed daemon server.");

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);

    let db = Arc::new(Mutex::new(HashMap::<String, Bytes>::new()));

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

    while let Some(message) = client_reader.next().await {
        info!("From {}: {:?}", addr, message);

        match message {
            Ok(InboundMessageType::Plate { plate, timestamp }) => {
                handle_plate(&plate, timestamp, conn.clone()).await?
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
                handle_i_am_camera(road, mile, limit, &tx, conn.clone()).await?;
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

async fn handle_plate(plate: &str, timestamp: u32, conn: Connection) -> anyhow::Result<()> {
    #[derive(Debug)]
    struct Road {
        speed_limit: i32,
    }

    #[derive(Debug)]
    struct Duration {
        time: i32,
    }

    #[derive(Debug)]
    struct Distance {
        miles: i32,
    }

    info!("Inserting plate: {} timestamp: {}", plate, timestamp);

    // let plate = std::sync::Arc::new(std::sync::Mutex::new(plate));

    // let plate = plate.clone();
    conn.call(move |conn| {
        // let plate = plate.lock().unwrap();
        conn.execute(
            "INSERT INTO plates (plate, timestamp) VALUES (?1, ?2)",
            params![&plate, timestamp],
        )
    })
    .await?;

    // Get the speed limit for the current road reported by this camera
    let speed_limits = conn
        .call(|conn| {
            let mut stmt = conn.prepare("SELECT speed_limit FROM cameras LIMIT 1")?;
            let speed_limits = stmt
                .query_map([], |row| {
                    Ok(Road {
                        speed_limit: row.get(0)?,
                    })
                })?
                .collect::<Result<Vec<Road>, rusqlite::Error>>()?;
            Ok::<_, rusqlite::Error>(speed_limits)
        })
        .await?;

    // Since we said "LIMIT 1" we only ever get one value, hence [0]
    info!("This road's speed limit is {}", speed_limits[0].speed_limit);

    // Get travel time.
    // This query selects the last two values from PLATES table using two subqueries with ORDER BY and LIMIT clauses,
    // then calculates the difference between them in the outer query using a simple "-" expression.
    // The JOIN on 1=1 is used to ensure that the query returns only a single row with the difference value.

    // NOTE: if there are fewer than two entries in the table, this query will not return any rows.
    let travel_times_sql = "SELECT (t2.timestamp - t1.timestamp) AS difference
        FROM (
            SELECT timestamp
            FROM plates
            WHERE plate=:plate
            ORDER BY timestamp DESC
            LIMIT 2
        ) t1
        JOIN (
            SELECT timestamp
            FROM plates
            WHERE plate=:plate
            ORDER BY timestamp DESC
            LIMIT 2
            OFFSET 1
        ) t2 ON 1=1
    ";

    // Get distance traveled.
    let distances_traveled_sql = "SELECT (t2.mile - t1.mile) AS difference
    FROM (
        SELECT mile
        FROM cameras
        ORDER BY mile DESC
        LIMIT 2
    ) t1
    JOIN (
        SELECT mile
        FROM cameras
        ORDER BY mile DESC
        LIMIT 2
        OFFSET 1
    ) t2 ON 1=1
    ";

    let travel_times = conn
        .call(move |conn| {
            let mut stmt = conn.prepare(travel_times_sql)?;
            let travel_times = stmt
                .query_map([], |row| Ok(Duration { time: row.get(0)? }))?
                .collect::<Result<Vec<Duration>, rusqlite::Error>>()?;
            Ok::<_, rusqlite::Error>(travel_times)
        })
        .await?;

    let distances_traveled = conn
        .call(move |conn| {
            let mut stmt = conn.prepare(distances_traveled_sql)?;
            let distances_traveled = stmt
                .query_map([], |row| Ok(Distance { miles: row.get(0)? }))?
                .collect::<Result<Vec<Distance>, rusqlite::Error>>()?;
            Ok::<_, rusqlite::Error>(distances_traveled)
        })
        .await?;

    info!(
        "Travel time is {} distance traveled is {}",
        travel_times[0].time, distances_traveled[0].miles
    );

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
    road: u16,
    mile: u16,
    speed_limit: u16,
    tx: &mpsc::Sender<OutboundMessageType>,
    conn: Connection,
) -> anyhow::Result<()> {
    info!(
        "Inserting camera road: {} mile: {} limit: {}",
        road, mile, speed_limit
    );

    let insert_query = "INSERT INTO cameras (road, mile, speed_limit) VALUES (?1, ?2, ?3)";

    // NOTE: there is one speed limit per road
    conn.call(move |conn| conn.execute(insert_query, params![road, mile, speed_limit]))
        .await?;

    Ok(())
}

fn handle_i_am_dispatcher(message: InboundMessageType) {
    todo!()
}

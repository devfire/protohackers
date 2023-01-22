// use std::error;
// use std::io;
use anyhow::Result;
use log::{error, info};
use rusqlite::Connection;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::io::{self, AsyncReadExt};
use tokio::net::TcpListener;

// https://doc.rust-lang.org/std/primitive.i32.html#method.from_be_bytes
fn read_be_i32(input: &[u8]) -> i32 {
    i32::from_be_bytes(input[..4].try_into().unwrap())
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    let listener = TcpListener::bind(&addr).await?;

    let create_table_query =
        "CREATE TABLE messages (id INTEGER PRIMARY KEY, timestamp INTEGER, price INTEGER)";

    while let Ok((stream, addr)) = listener.accept().await {
        info!("New connection from {}", addr);

        info!("Creating the table.");

        let connection = Connection::open_in_memory()?;

        connection
            .execute(create_table_query, ())
            .expect("Failed to create table");

        let stream = stream;
        let (reader, mut _writer) = tokio::io::split(stream);
        let mut reader = io::BufReader::new(reader);

        loop {
            let mut buffer = [0; 9];
            let n = reader
                .read_exact(&mut buffer)
                .await
                .expect("Unable to read from buffer");
            println!("Bytes received: {} buffer length: {}", n, buffer.len());
            if n == 0 {
                break;
            }

            // (0..9).for_each(|i| {
            //     print!("byte #{}, {:#010b} ",i, &buffer[i]);
            // });

            // The message format is:
            // Byte:  |  0  |  1     2     3     4  |  5     6     7     8  |
            // Type:  |char |         int32         |         int32         |
            // Value: | 'I' |       timestamp       |         price         |
            //

            let msg_type = &buffer[0];

            // Read slices 1,2,3,4.
            // Since we have a slice rather than an array, fallible conversion APIs can be used
            let first_half_decoded = read_be_i32(&buffer[1..=4]);
            let second_half_decoded = read_be_i32(&buffer[5..n]);

            println!(
                "Type: {}, first: {}, second: {}",
                msg_type, first_half_decoded, second_half_decoded
            );

            match msg_type {
                73 => drop(
                    connection
                        .execute(
                            "INSERT INTO messages (timestamp, price) VALUES (?1, ?2)",
                            (first_half_decoded, second_half_decoded),
                        )
                        .expect("Unable to insert data"),
                ),
                81 => drop(
                    connection
                        .execute(
                            "SELECT AVG(price) FROM messages WHERE timestamp BETWEEN ?1 AND ?2",
                            (first_half_decoded, second_half_decoded),
                        )
                        .expect("Unable to query data"),
                ),
                _ => error!("Fail"),
            }
        }
    }

    Ok(())
}

// use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
// use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, AsyncReadExt};
use tokio::io::{self, AsyncReadExt};
use tokio::net::TcpListener;
use sqlite::State;



// https://doc.rust-lang.org/std/primitive.i32.html#method.from_be_bytes
fn read_be_i32(input: &[u8]) -> i32 {
    i32::from_be_bytes(input[..4].try_into().unwrap())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    let listener = TcpListener::bind(&addr).await?;

    while let Ok((stream, addr)) = listener.accept().await {
        println!("New connection from {}", addr);

        let stream = stream;
        let (reader, mut _writer) = tokio::io::split(stream);
        let mut reader = io::BufReader::new(reader);

        tokio::spawn(async move {
            loop {
                let mut buffer = [0;9];
                let n = reader.read_exact(&mut buffer).await.expect("Unable to read buffer");
                println!("Bytes received: {} buffer length: {}", n, buffer.len());
                if n == 0 {
                    break;
                }

                // (0..9).for_each(|i| {
                //     print!("byte #{}, {:#010b} ",i, &buffer[i]);
                // });

                let msg_type = &buffer[0];

                // read slices 1,2,3,4
                let first_half_decoded = read_be_i32(&buffer[1..=4]);
                let second_half_decoded = read_be_i32(&buffer[5..n]);

                println!("Type: {}, first: {}, second: {}", msg_type, first_half_decoded, second_half_decoded);

                match msg_type {
                    73 => insert_data(first_half_decoded, second_half_decoded),
                    81 => query_data(first_half_decoded, second_half_decoded),
                    _ => println!("Got something weird, ignoring.")
                }
            }
        });
    }

    Ok(())
}
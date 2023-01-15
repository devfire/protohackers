// use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
// use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, AsyncReadExt};
use tokio::io::{self, AsyncReadExt};
use tokio::net::TcpListener;

// enum MsgType {
//     I,
//     Q,
// }

// fn read_be_i32(input: &mut &[u8]) -> i32 {
//     let (int_bytes, rest) = input.split_at(std::mem::size_of::<i32>());
//     *input = rest;
//     i32::from_be_bytes(int_bytes.try_into().expect("Failed to convert from slice to array"))
// }


#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    let listener = TcpListener::bind(&addr).await?;

    while let Ok((stream, addr)) = listener.accept().await {
        println!("New connection from {}", addr);

        let stream = stream;
        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = io::BufReader::new(reader);

        tokio::spawn(async move {
            loop {
                let mut buffer = [0;9];
                let n = reader.read_exact(&mut buffer).await.expect("Unable to read buffer");
                println!("Bytes received: {}", n);
                if n == 0 {
                    break;
                }
                println!("The bytes: {:?}", &buffer[..n]);

                let msg_type = buffer[0];
                // let first_half: i32 = read_be_i32(&mut &buffer[1..4]);
                // let second_half: i32 = read_be_i32(&mut &buffer[5..8]);
                
                let first_half = i32::from_be_bytes(buffer[1..4].try_into().expect("from slice to array failed"));
                let second_half = i32::from_be_bytes(buffer[5..].try_into().expect("from slice to array failed"));

                println!("Type: {}, first_half: {}, second_half {}", msg_type, first_half, second_half);
            }
        });
    }

    Ok(())
}
// use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
// use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, AsyncReadExt};
use tokio::io::{self, AsyncReadExt};
use tokio::net::TcpListener;

// https://doc.rust-lang.org/std/primitive.i32.html#method.from_be_bytes
// fn read_be_i32(input: &mut &[u8]) -> i32 {
//     let (int_bytes, rest) = input.split_at(std::mem::size_of::<i32>());
//     *input = rest;
//     i32::from_be_bytes(int_bytes.try_into().expect("Failed in read_be_i32 to convert from slice to array"))
// }

fn make_array(input: &[u8]) -> [u8;4] {
    input.try_into().expect("slice with incorrect length")
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

                (0..9).for_each(|i| {
                    print!("byte #{}, {:#010b} ",i, &buffer[i]);
                });

                let msg_type = &buffer[0];
                let first_half_decoded = i32::from_be_bytes(make_array(&buffer[1..4]));
                let second_half_decoded = i32::from_be_bytes(make_array(&buffer[5..8]));

                // let first_half_decoded = read_be_i32(&mut &buffer[1..4]);
                // let second_half_decoded = read_be_i32(&mut &buffer[5..8]);

                println!("Type: {}, first: {}, second: {}", msg_type, first_half_decoded, second_half_decoded);
            }
        });
    }

    Ok(())
}
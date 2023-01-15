// use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
// use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, AsyncReadExt};
use tokio::io::{self, AsyncReadExt};
use tokio::net::TcpListener;

// enum MsgType {
//     I,
//     Q,
// }

fn read_be_i32(input: &mut &[u8]) -> i32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<i32>());
    *input = rest;
    i32::from_be_bytes(int_bytes.try_into().expect("Failed in read_be_i32 to convert from slice to array"))
}


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
                println!("Bytes received: {} buffer length: {}", n, buffer.len());
                if n == 0 {
                    break;
                }

                (0..9).for_each(|i| {
                    print!("byte #{}, {:#010b} ",i, &buffer[i]);
                });

                println!();


                // println!("The bytes: {:b}", &buffer[..n]);

                let msg_type = &buffer[0];
                let mut first_half = &buffer[1..4];
                let mut second_half = &buffer[5..8];

                // let second_half: i32 = read_be_i32(&mut &buffer[5..8]);
                
                // let first_half_decoded = i32::from_be_bytes(first_half.try_into().expect("from slice to array failed"));
                // let second_half_decoded = i32::from_be_bytes(second_half.try_into().expect("from slice to array failed"));

                let first_half_decoded = read_be_i32(&mut first_half);
                let second_half_decoded = read_be_i32(&mut second_half);


                println!("Type: {}, first: {}, second: {}", msg_type, first_half_decoded, second_half_decoded);
            }
        });
    }

    Ok(())
}
// use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpListener;

enum MsgType {
    I,
    Q,
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
                let mut buffer = [0; 9];
                let n = reader.read_exact(&mut buffer).await.expect("Unable to read buffer");
                if n == 0 {
                    break;
                }
                println!("The bytes: {:?}", &buffer[..n]);
                // let msg_type_ascii = buffer[0];

                let msg_type = buffer[0];

                let first_half: String = format!("{:b}{:b}{:b}{:b}", buffer[1], buffer[2], buffer[3], buffer[4]);

                let second_half: String = format!("{}{}{}{}", buffer[5], buffer[6], buffer[7], buffer[8]);

                println!("Type: {}, first_half: {}, second_half {}", msg_type, first_half, second_half);
            }
        });
    }

    Ok(())
}
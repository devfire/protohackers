use std::error::Error;
use std::net::{TcpListener, TcpStream};
use tokio::io::{copy, AsyncReadExt, AsyncWriteExt};
// use tokio::net::TcpStream as AsyncTcpStream;
// use tokio::task::futures;
// use tokio::prelude::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await;

    loop {
        let (client, _) = listener.accept().await?;
        tokio::spawn(async move {
            let server = TcpStream::connect("google.com:80").await?;
            let (client_reader, client_writer) = client.split();
            let (server_reader, server_writer) = server.split();

            let server_to_client = copy(server_reader, client_writer);
            let client_to_server = async move {
                let mut buf = [0u8; 1024];
                while let Ok(n) = server_reader.read(&mut buf[..]).await {
                    if n == 0 {
                        return Ok(());
                    }
                    client_writer.write_all(&buf[..n]).await?;
                    // client_writer.write_all(b".").await?;
                }
                Ok(())
            };

            tokio::join!(client_to_server, server_to_client);

            Ok(())
        });
    }
}

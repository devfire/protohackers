use std::net::SocketAddr;

use env_logger::Env;
use log::{error, info};

use fancy_regex::Regex;
use futures::{SinkExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};

use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

use anyhow::Result;

const TONYCOIN: &str = "7YWHMfk9JZe0LM0g1ZauHuiSxhI";

#[tokio::main]
async fn main() -> Result<()> {
    // Setup the logging framework
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    info!("Starting the proxy server.");
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    info!("Ready to steal crypto!");

    // Accept incoming connections
    while let Ok((client, addr)) = listener.accept().await {
        // Spawn our handler to be run asynchronously.

        info!("accepted connection from {}", addr);
        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            info!("accepted connection from {}", addr);
            if let Err(e) = process(client, addr).await {
                error!("an error occurred; error = {:?}", e);
            }
        });
    }

    Ok(())
}

/// Defines a new asynchronous function `process` that takes two arguments:
/// `client`, a mutable reference to a TcpStream, and `server_addr`, a string slice of the remote server.
async fn process(client_stream: TcpStream, client_addr: SocketAddr) -> Result<()> {
    let (client_reader, client_writer) = client_stream.into_split();
    let mut client_reader = FramedRead::new(client_reader, LinesCodec::new());
    let mut client_writer = FramedWrite::new(client_writer, LinesCodec::new());

    let server_stream = TcpStream::connect("chat.protohackers.com:16963").await?;
    let (server_reader, server_writer) = server_stream.into_split();
    let mut server_reader = FramedRead::new(server_reader, LinesCodec::new());
    let mut server_writer = FramedWrite::new(server_writer, LinesCodec::new());

    let _client_task = tokio::spawn(async move {
        let re = Regex::new(r"(?<=\A| )7[A-Za-z0-9]{25,35}(?=\z| )").unwrap();
        while let Some(message) = client_reader.next().await {
            match message {
                Ok(message) => {
                    let replaced = re.replace_all(&message, TONYCOIN);
                    info!("Client {} to server: {}", client_addr, replaced);
                    server_writer.send(&replaced).await?;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok(())
    });

    let _server_task = tokio::spawn(async move {
        let re = Regex::new(r"(?<=\A| )7[A-Za-z0-9]{25,35}(?=\z| )").unwrap();
        while let Some(message) = server_reader.next().await {
            match message {
                Ok(message) => {
                    let replaced = re.replace_all(&message, TONYCOIN);
                    info!("Message: {} to client: {}", replaced, client_addr);
                    client_writer.send(&replaced).await?;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok(())
    });

    Ok(())
}

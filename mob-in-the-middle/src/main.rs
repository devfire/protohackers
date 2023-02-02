use std::net::SocketAddr;

use env_logger::Env;
use log::{error, info};

use fancy_regex::Regex;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

// use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

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
async fn process(client_stream: TcpStream, addr: SocketAddr) -> Result<()> {
    let server_stream = TcpStream::connect("chat.protohackers.com:16963").await?;

    let (server_reader, mut server_writer) = tokio::io::split(server_stream);
    let (client_reader, mut client_writer) = tokio::io::split(client_stream);

    let mut client_reader = tokio::io::BufReader::new(client_reader);
    let mut server_reader = tokio::io::BufReader::new(server_reader);

    tokio::spawn(async move {
        //Explanation of the regex:
        // (?<=\A| ): Matches either the start of the message (\A) or a space character ( ), lookbehind assertion
        // 7: Matches the character 7 literally
        // [A-Za-z0-9]{26,35}: Matches 25 to 35 alphanumeric characters (A-Za-z0-9)
        // (?=\z| ): Matches either the end of the message (\z) or a space character ( ), lookahead assertion
        let re = Regex::new(r"(?<=\A| )7[A-Za-z0-9]{25,35}(?=\z| )").unwrap();
        let mut data = String::new();
        loop {
            let n = match client_reader.read_line(&mut data).await {
                Ok(n) => n,
                Err(e) => {
                    error!("Error forwarding data from client: {}", e);
                    break;
                }
            };

            if n == 0 {
                break;
            }

            // The method returns a new string with the matches replaced.
            let replaced = re.replace_all(&data, TONYCOIN);

            info!("Client {} to server: {}",addr, replaced);

            server_writer
                .write_all(replaced.as_bytes())
                .await
                .expect("Sending to server failed");
        }
    });

    
    tokio::spawn(async move {
        let re = Regex::new(r"(?<=\A| )7[A-Za-z0-9]{25,35}(?=\z| )").unwrap();
        let mut data = String::new();
        loop {
            let n = match server_reader.read_line(&mut data).await {
                Ok(n) => n,
                Err(e) => {
                    error!("Error forwarding data from server: {}", e);
                    break;
                }
            };

            if n == 0 {
                break;
            }

            // If no match, the string is returned intact.
            let replaced = re.replace_all(&data, TONYCOIN);

            info!("Server to client {}: {}", addr, replaced);

            client_writer
                .write_all(replaced.as_bytes())
                .await
                .expect("Sending to client failed");
        }
    });

    Ok(())
}

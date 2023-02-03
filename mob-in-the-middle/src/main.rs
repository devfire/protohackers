use std::net::SocketAddr;

use env_logger::Env;
use log::{error, info};

use fancy_regex::Regex;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup the logging framework
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    info!("Starting the proxy server.");
    // Bind the listener to the address
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    info!("Ready to steal crypto!");

    // Accept incoming connections
    while let Ok((client_stream, client_addr)) = listener.accept().await {
        // Spawn a task to handle each client
        tokio::spawn(process(client_stream, client_addr, "chat.protohackers.com:16963"));
    }

    Ok(())
}

/// Defines a new asynchronous function `process` that takes two arguments:
/// `client`, a mutable reference to a TcpStream, and `server_addr`, a string slice of the remote server.
async fn process(client_stream: TcpStream, client_addr: SocketAddr ,server_addr: &str) -> Result<()> {
    info!(
        "Establishing a connection to the upstream server on behalf of {}.",
        server_addr
    );
    let server_stream = TcpStream::connect(server_addr).await?;

    let (mut server_reader, mut server_writer) = tokio::io::split(server_stream);
    let (mut client_reader, mut client_writer) = tokio::io::split(client_stream);

    let client_to_server = async move {
        let re = Regex::new(r"(?<=\A| )7[A-Za-z0-9]{25,35}(?=\z| )").unwrap();
        let mut client_buf = [0; 1024];
        loop {
            let n = match client_reader.read(&mut client_buf).await {
                Ok(n) => n,
                Err(e) => {
                    error!("Error forwarding data from client: {}", e);
                    break;
                }
            };

            if n == 0 {
                break;
            }

            let data = String::from_utf8(client_buf[..n].to_vec()).unwrap();
            let replaced = re.replace_all(&data, "7YWHMfk9JZe0LM0g1ZauHuiSxhI");

            info!("{} -> {}",client_addr, replaced.trim_end());

            server_writer
                .write_all(replaced.as_bytes())
                .await
                .expect("Sending to server failed");
        }
    };

    let server_to_client = async move {
        let re = Regex::new(r"(?<=\A| )7[A-Za-z0-9]{25,35}(?=\z| )").unwrap();
        let mut server_buf = [0; 1024];

        loop {
            let n = match server_reader.read(&mut server_buf).await {
                Ok(n) => n,
                Err(e) => {
                    error!("Error forwarding data from server: {}", e);
                    break;
                }
            };

            if n == 0 {
                break;
            }

            let data = String::from_utf8(server_buf[..n].to_vec()).unwrap();

            // re.replace method takes two arguments:
            // the original string and the string to replace the match with.
            // The method returns a new string with the matches replaced.
            // If no match, the string is returned intact.
            let replaced = re.replace_all(&data, "7YWHMfk9JZe0LM0g1ZauHuiSxhI");

            info!("To: {} ->{}",client_addr, replaced.trim_end());
            client_writer
                .write_all(replaced.as_bytes())
                .await
                .expect("Sending to server failed");
        }
    };

    let (_, _) = tokio::join!(client_to_server, server_to_client);

    Ok(())
}

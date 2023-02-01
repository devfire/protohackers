use env_logger::Env;
use log::{error, info};

use tokio::{
    io::{copy, AsyncReadExt, AsyncWriteExt},
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
    while let Ok((client, _)) = listener.accept().await {
        // Spawn a task to handle each client
        tokio::spawn(process(client, "chat.protohackers.com:16963"));
    }

    Ok(())
}

/// Defines a new asynchronous function `process` that takes two arguments:
/// `client`, a mutable reference to a TcpStream, and `server_addr`, a string slice of the remote server.
async fn process(client_stream: TcpStream, server_addr: &str) -> Result<()> {
    info!(
        "Establishing a connection to the upstream server on behalf of {}.",
        server_addr
    );
    let server_stream = TcpStream::connect("chat.protohackers.com:16963").await?;
    info!("Connection established.");

    let (mut server_reader, mut server_writer) = tokio::io::split(server_stream);
    // let mut server_reader = io::BufReader::new(server_reader);

    let (mut client_reader, mut client_writer) = tokio::io::split(client_stream);
    // let mut client_reader = io::BufReader::new(client_reader);

    let client_to_server = async move {
        let mut buf = [0; 1024];

        loop {
            let n = match client_reader.read(&mut buf).await {
                Ok(n) => n,
                Err(e) => {
                    error!("Error forwarding data from client: {}", e);
                    break;
                }
            };
            info!("Received {} bytes from client", n);

            if n == 0 {
                break;
            }

            let data = String::from_utf8(buf[..n].to_vec())
                .unwrap()
                .replace(r"7[a-zA-Z0-9]{25,34}", "7YWHMfk9JZe0LM0g1ZauHuiSxhI")
                .into_bytes();

            server_writer.write_all(&data).await.unwrap();
        }
    };

    let server_to_client = copy(&mut server_reader, &mut client_writer);

    let (_, _) = tokio::join!(client_to_server, server_to_client);

    Ok(())
}

use std::net::SocketAddr;

use regex::Regex;

use env_logger::Env;
use log::{error, info};

use tokio::{
    io::{self, copy, AsyncBufReadExt, AsyncWriteExt},
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

    loop {
        // Asynchronously wait for an inbound TcpStream.
        let (client_facing_stream, addr) = listener.accept().await?;

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            info!("accepted connection from {}", addr);
            if let Err(e) = process(client_facing_stream, addr).await {
                error!("an error occurred; error = {:?}", e);
            }
        });
    }
}

async fn process(to_client_stream: TcpStream, addr: SocketAddr) -> Result<()> {
    info!(
        "Establishing a connection to the upstream server on behalf of {}.",
        addr
    );
    let to_server_stream = TcpStream::connect("chat.protohackers.com:16963").await?;
    info!("Connection established.");

    let (mut server_reader, mut server_writer) = tokio::io::split(to_server_stream);
    // let mut server_reader = io::BufReader::new(server_reader);

    let (mut client_reader, mut client_writer) = tokio::io::split(to_client_stream);
    // let mut client_reader = io::BufReader::new(client_reader);

    let client_to_server = copy(&mut client_reader, &mut server_writer);
    let server_to_client = copy(&mut server_reader, &mut client_writer);

    tokio::join!(client_to_server, server_to_client);

    Ok(())
}

fn replace_substring(s: &str, old: &str, new: &str) -> String {
    s.replace(old, new)
}

fn get_substring(s: &str) -> Option<&str> {
    let re = Regex::new(r"^ 7[a-zA-Z0-9]{25,34} $").unwrap();
    re.find(s).map(|m| m.as_str())
}

fn steal_crypto(line: &str) -> String {
    if let Some(boguscoin_address) = get_substring(line) {
        replace_substring(line, boguscoin_address, "7YWHMfk9JZe0LM0g1ZauHuiSxhI")
    } else {
        String::from(line)
    }
}

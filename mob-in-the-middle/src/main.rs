use std::net::SocketAddr;

use lazy_static::lazy_static;

use env_logger::Env;
use log::info;
use regex::Regex;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

use anyhow::{bail, Result};

async fn read_next_line(r: &mut (impl AsyncBufReadExt + Unpin)) -> Result<String> {
    let mut line = String::new();
    if 0 == r.read_line(&mut line).await? {
        bail!("no message");
    }
    line = hack_line(&line);
    Ok(line)
}

fn hack_line(line: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"(\b7[[:alnum:]]{25,34})(\s|$)"#).unwrap();
    }

    RE.replace_all(line, |caps: &regex::Captures| {
        format!("{}{}", "7YWHMfk9JZe0LM0g1ZauHuiSxhI", &caps[2])
    })
    .to_string()
}

async fn write_next_line(w: &mut (impl AsyncWriteExt + Unpin), msg: &str) -> Result<()> {
    w.write_all(msg.as_bytes()).await?;
    Ok(w.flush().await?)
}


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
        tokio::spawn(process(
            client_stream,
            client_addr,
            "chat.protohackers.com:16963",
        ));
    }

    Ok(())
}

/// Defines a new asynchronous function `process` that takes two arguments:
/// `client`, a mutable reference to a TcpStream, and `server_addr`, a string slice of the remote server.
async fn process(
    client_stream: TcpStream,
    client_addr: SocketAddr,
    server_addr: &str,
) -> anyhow::Result<()> {
    info!(
        "Establishing a connection to the upstream server on behalf of {}.",
        server_addr
    );
    let server_stream: TcpStream = TcpStream::connect(server_addr).await?;

    let (server_reader, mut server_writer) = server_stream.into_split();
    let (client_reader, mut client_writer) = client_stream.into_split();

    let mut server_reader = BufReader::new(server_reader);
    let mut client_reader = BufReader::new(client_reader);

    tokio::spawn({
        async move {
            loop {
                if let Ok(server_line) = read_next_line(&mut server_reader).await {
                    info!("From {}->{}", client_addr, server_line);
                    let _ = write_next_line(&mut client_writer, &server_line).await;
                }
            }
        }
    });

    loop {
        let client_line = read_next_line(&mut client_reader).await?;
        info!("To {}->{}", client_addr, client_line);
        write_next_line(&mut server_writer, &client_line).await?;
    }
}

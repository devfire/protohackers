use std::net::SocketAddr;

use env_logger::Env;
use fancy_regex::Regex;
use log::{info, warn};
use tokio::select;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
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
    mut client_stream: TcpStream,
    client_addr: SocketAddr,
    server_addr: &str,
) -> anyhow::Result<()> {
    info!(
        "Establishing a connection to the upstream server on behalf of {}.",
        server_addr
    );
    let mut server_stream: TcpStream = TcpStream::connect(server_addr).await?;

    let (server_reader, mut server_writer) = server_stream.split();
    let (client_reader, mut client_writer) = client_stream.split();

    let mut server_reader = BufReader::new(server_reader);
    let mut client_reader = BufReader::new(client_reader);

    let client_to_server = async {
        let re = Regex::new(r"(?<=\A| )7[A-Za-z0-9]{25,35}(?=\z| )").unwrap();
        let mut client_line = String::with_capacity(1024);
        loop {
            client_line.clear();
            let bytes_read = client_reader
                .read_line(&mut client_line)
                .await
                .expect("Unable to read from client");

            if bytes_read == 0 {
                warn!("EOF");
                break;
            }

            if !client_line.ends_with('\n') {
                warn!("Disconnected without sending \\n");
                break;
            }

            // let data = String::from_utf8(client_buf[..n].to_vec()).unwrap();
            let client_line = re.replace_all(&client_line, "7YWHMfk9JZe0LM0g1ZauHuiSxhI");

            info!("{} -> {}", client_addr, client_line);

            server_writer
                .write_all(format!("{}\n", client_line).as_bytes())
                .await
                .expect("Sending to server failed");
            server_writer.flush().await.expect("Unable to flush");
        }
        info!("Client {} disconnected.", client_addr);
        Ok::<(), anyhow::Error>(())
    };

    let server_to_client = async {
        let re = Regex::new(r"(?<=\A| )7[A-Za-z0-9]{25,35}(?=\z| )").unwrap();
        let mut server_line = String::with_capacity(1024);

        loop {
            server_line.clear();
            let bytes_read = server_reader
                .read_line(&mut server_line)
                .await
                .expect("Unable to read server");

            // if bytes_read == 0 {
            //     warn!("EOF");
            //     break;
            // }
            if !server_line.ends_with('\n') {
                warn!("Disconnected without sending \\n");
                break;
            }

            let server_line = re.replace_all(&server_line, "7YWHMfk9JZe0LM0g1ZauHuiSxhI");

            info!("To: {} ->{}", client_addr, server_line);
            client_writer
                .write_all(format!("{}\n", server_line).as_bytes())
                .await
                .expect("Sending to server failed");
            client_writer
                .flush()
                .await
                .expect("Unable to flush to client");
        }
        info!("Server to client disconnected.");
        Ok::<(), anyhow::Error>(())
    };

    select! {
        _ = client_to_server => {},
        _ = server_to_client => {},
    }

    info!("Disconnect");

    Ok(())
}

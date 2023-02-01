use env_logger::Env;
use log::{error, info};

use fancy_regex::Regex;

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
    // info!("Connection established.");

    let (mut server_reader, mut server_writer) = tokio::io::split(server_stream);
    // let mut server_reader = io::BufReader::new(server_reader);

    let (mut client_reader, mut client_writer) = tokio::io::split(client_stream);
    // let mut client_reader = io::BufReader::new(client_reader);

    let re = Regex::new(r"(?<=\A| )7[A-Za-z0-9]{25,35}(?=\z| )").unwrap();

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

            // Convert a buffer to a string by using the String::from_utf8 function.
            // This function takes a Vec<u8> as its argument and returns a Result<String, Utf8Error>.
            // If the buffer contains valid UTF-8 encoded data, the Result will be Ok with the resulting string,
            // otherwise the Result will be Err with a Utf8Error indicating the first invalid byte.
            let from_client = String::from_utf8(buf.to_vec()).expect("Buffer to string failed");
            info!("Received from client: {}", from_client.trim_end());

            if n == 0 {
                break;
            }

            let data = String::from_utf8(buf[..n].to_vec()).unwrap();

            // re.replace method takes two arguments: 
            // the original string and the string to replace the match with. 
            // The method returns a new string with the matches replaced.
            let replaced = re.replace_all(&data, "7YWHMfk9JZe0LM0g1ZauHuiSxhI");

            info!("Replaced: {}", replaced);

            // let to_server = String::from_utf8(replaced.as_bytes().to_vec()).expect("Buffer to string failed");

            server_writer
                .write_all(replaced.as_bytes())
                .await
                .expect("Sending to server failed");
        }
    };

    let server_to_client = copy(&mut server_reader, &mut client_writer);

    let (_, _) = tokio::join!(client_to_server, server_to_client);

    Ok(())
}

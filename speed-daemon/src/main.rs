use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use env_logger::Env;
use log::{error, info};

use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup the logging framework
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    // Bind the listener to the address
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    loop {
        // The second item contains the ip and port of the new connection.
        let (client_stream, client_addr) = listener.accept().await?;

        info!("New connection from {}", client_addr);

        // A new task is spawned for each inbound socket.  The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(client_stream)
                .await
                .expect("Failed to spawn a new processor");
        });
    }
}

async fn process(stream: TcpStream) -> anyhow::Result<()> {
    let (mut reader, mut writer) = tokio::io::split(stream);

    loop {
        //Each message starts with a single u8 specifying the message type.
        let mut msg_type = [1];

        let n = reader.read_exact(&mut msg_type).await?;
    }
}

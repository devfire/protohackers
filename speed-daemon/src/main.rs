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
        let (client_stream, addr) = listener.accept().await?;

        info!("New connection from {}", addr);

        // A new task is spawned for each inbound socket.  The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(client_stream).await;
        });
    }
}

async fn process(stream: TcpStream) {
    let (mut reader, mut writer) = tokio::io::split(stream);

    loop {
        //declare a buffer precisely 9 bytes long
        let mut msg_type: u8 = 0x0;
    }
}

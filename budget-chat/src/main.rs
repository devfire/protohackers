use log::info;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

extern crate log;

use env_logger::Env;

struct Users {
    username: String,
    joined: bool,
}

#[tokio::main]
async fn main() {
    let env = Env::default()
        .filter_or("LOG_LEVEL", "trace")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    info!("Starting the chat server.");

    // Bind the listener to the address
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    while let Ok((stream, addr)) = listener.accept().await {
        info!("New connection from {}", addr);

        // A new task is spawned for each inbound socket.  The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(stream).await;
        });
    }
}

fn get_username(mut reader: io::BufReader<io::ReadHalf<TcpStream>>) -> String {
    let mut line = String::new();
    let _n = reader.read_line(&mut line);

    info!("Username received: {:?}", line);
    line
}
async fn process(stream: TcpStream) {
    let (reader, mut writer) = tokio::io::split(stream);

    let reader: io::BufReader<io::ReadHalf<TcpStream>> = io::BufReader::new(reader);

    let mut users: Vec<Users>;

    let new_username = get_username(reader);

    // notify_presence
}

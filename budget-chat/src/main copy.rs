use log::info;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

use futures::SinkExt;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io;
// use std::net::SocketAddr;
use std::sync::Arc;

use env_logger::Env;

extern crate log;

#[tokio::main]
async fn main() {
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    info!("Starting the chat server.");

    // Bind the listener to the address
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    
    // Create the shared state. This is how all the peers communicate.
    //
    // The server task will hold a handle to this. For every new client, the
    // `state` handle is cloned and passed into the task that processes the
    // client connection.
    let state = Arc::new(Mutex::new(Shared::new()));

    while let Ok((stream, addr)) = listener.accept().await {
        // Clone the handle to the db
        let db = users_db.clone();

        info!("New connection from {}", addr);

        // A new task is spawned for each inbound socket.  The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(stream, db).await;
        });
    }
}

fn get_username(mut reader: io::BufReader<io::ReadHalf<TcpStream>>) -> String {
    let mut line = String::new();
    let _n = reader.read_line(&mut line);

    info!("Username received: {:?}", line);
    line
}

fn validate_username(username: &String) -> bool {
    // make sure the username is at least 1 char,
    // and consists of alphanumeric chars only.
    // Return FALSE if username is empty OR not alphanumeric.
    username.is_empty() || !username.chars().all(char::is_alphanumeric)
}


async fn process(stream: TcpStream, db: Db) {
    let (reader, writer) = tokio::io::split(stream);

    let reader: io::BufReader<io::ReadHalf<TcpStream>> = io::BufReader::new(reader);

    let new_username = get_username(reader);

    // Before we "join" the user let's make sure they passed a valid username
    if validate_username(&new_username) {
        info!("Username {} is valid, adding.", new_username);
        let mut db = db.lock().unwrap();
        db.push(new_username);
    }


    loop {

        // notify_presence
    } // end of loop
}

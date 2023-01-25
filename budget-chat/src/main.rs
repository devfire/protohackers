// heavily borrowed from https://raw.githubusercontent.com/tokio-rs/tokio/master/examples/chat.rs
#![warn(rust_2018_idioms)]

use futures::SinkExt;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

use std::{error::Error, sync::Arc};

use budget_chat::{Peer, Shared};

use env_logger::Env;
use log::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup the logging framework
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    // Create the shared state. This is how all the peers communicate.
    //
    // The server task will hold a handle to this. For every new client, the
    // `state` handle is cloned and passed into the task that processes the
    // client connection.
    let state = Arc::new(Mutex::new(Shared::new()));

    info!("Starting the chat server.");

    // Bind the listener to the address
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    info!("Server is running.");

    loop {
        // Asynchronously wait for an inbound TcpStream.
        let (stream, addr) = listener.accept().await?;

        // Clone a handle to the `Shared` state for the new connection.
        let state = Arc::clone(&state);

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            info!("accepted connection from {}", addr);
            if let Err(e) = process(state, stream, addr).await {
                info!("an error occurred; error = {:?}", e);
            }
        });
    }
}

// make sure the username is at least 1 char,
// and consists of alphanumeric chars only.
// Return FALSE if username is empty OR not alphanumeric.
fn valid_username(username: &String) -> bool {
    username.is_empty() || !username.chars().all(char::is_alphanumeric)
}

/// Process an individual chat client
async fn process(
    state: Arc<Mutex<Shared>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut lines = Framed::new(stream, LinesCodec::new());

    // Send a prompt to the client to enter their username.
    lines
        .send("Welcome to budgetchat! What shall I call you? ")
        .await?;

    // Read the first line from the `LineCodec` stream to get the username.
    let username = match lines.next().await {
        Some(Ok(line)) => line,
        // We didn't get a line so we return early here.
        _ => {
            error!("Failed to get username from {}. Client disconnected.", addr);
            return Ok(());
        }
    };

    if !valid_username(&username) {
        error!("Invalid username: {}", username);
        return Ok(());
    }

    // Register our peer with state which internally sets up some channels.
    let mut peer = Peer::new(state.clone(), lines, username.clone()).await?;

    // A client has connected, let's let everyone know.
    {
        let mut state = state.lock().await;
        let msg = format!("* {} has entered the room.", username);
        info!("{}", msg);
        state.broadcast(addr, &msg).await;
    }

    // Publish all present users' names
    {
        let mut state = state.lock().await;
        let everyone_else = state.get_everyone(addr);

        let msg = format!("* The room contains: {:?}", everyone_else);
        info!("{}", msg);
        state.send_to_sender(addr, &msg);
    }

    // Process incoming messages until our stream is exhausted by a disconnect.
    loop {
        tokio::select! {
            // A message was received from a peer. Send it to the current user.
            Some(msg) = peer.rx.recv() => {
                peer.lines.send(&msg).await?;
            }
            result = peer.lines.next() => match result {
                // A message was received from the current user, we should
                // broadcast this message to the other users.
                Some(Ok(msg)) => {
                    let mut state = state.lock().await;
                    let msg = format!("[{}] {}", username, msg);
                    info!("{}", msg);
                    state.broadcast(addr, &msg).await;
                }
                // An error occurred.
                Some(Err(e)) => {
                    error!(
                        "an error occurred while processing messages for {}; error = {:?}",
                        username,
                        e
                    );
                }
                // The stream has been exhausted.
                None => break,
            },
        }
    }

    // If this section is reached it means that the client was disconnected!
    // Let's let everyone still connected know about it.
    {
        let mut state = state.lock().await;
        state.peers.remove(&addr);

        let msg = format!("* {} has left the room", username);
        info!("{}", msg);
        state.broadcast(addr, &msg).await;
    }

    Ok(())
}

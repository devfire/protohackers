use std::collections::HashMap;

use tokio::net::UdpSocket;

use env_logger::Env;
use log::info;

enum RequestType {
    Insert,
    Retrieve,
    Version,
}

/// This function returns the key & value pair
/// if this is an INSERT op, or None otherwise
/// https://doc.rust-lang.org/std/primitive.str.html#method.split_once
fn get_kv_pair(msg: &[u8]) -> Option<(String, String)> {
    String::from_utf8_lossy(msg)
        .split_once('=')
        .map(|(key, value)| -> (String, String) { (key.to_string(), value.to_string()) })
}

fn parse_request(msg: &[u8]) -> RequestType {
    if String::from_utf8_lossy(msg).starts_with("version") {
        RequestType::Version
    } else if String::from_utf8_lossy(msg).contains('=') {
        RequestType::Insert
    } else {
        RequestType::Retrieve
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup the logging framework
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);
    let socket = UdpSocket::bind("0.0.0.0:8080").await?;
    info!("Listening on {}", socket.local_addr()?);

    // main storage object for all the messages
    let mut db: std::collections::HashMap<String, String> = HashMap::new();

    loop {
        let mut buf = vec![0u8; 1024];
        let (size, peer) = socket.recv_from(&mut buf).await?;
        info!("Received message from {}", peer);

        let msg = &buf[..size];

        match parse_request(msg) {
            RequestType::Insert => {
                if let Some((k, v)) = get_kv_pair(msg) {
                    info!("Insert message type detected, adding {}={}", k, v);
                    db.insert(k, v);
                }
            }
            RequestType::Retrieve => {
                // OK, it's a Retrieve type, let's convert the message to a String
                // and pull the value from the HashMap
                let key_as_string =
                    String::from_utf8(msg.to_vec()).expect("utf8 to String conversion failed");

                // if this k,v exists, we send it back. If not, we go silent and ignore.
                if let Some(reply) = db.get(&key_as_string) {
                    info!("Retrieve message type detected, replying with {}", reply);
                    let amt = socket.send_to(reply.as_bytes(), &peer).await?;
                    info!("Sent {} bytes back to {}.", amt, peer);
                }
            }
            RequestType::Version => {
                let version = "version=budget DB 1.0";
                let amt = socket.send_to(version.as_bytes(), &peer).await?;
                info!("Sent version {} bytes back to {}.", amt, peer);
            }
        }
        // If this comes back with something, that means this was an Insert,
        // so we just add it to hashmap
        match get_kv_pair(msg) {
            Some((k, v)) => {
                info!("Insert message type detected, adding {}={}", k, v);
                db.insert(k, v);
            }
            None => {
                // OK, it's either a Retrieve type or a Version, let's see if it's a version.

                // let's convert the message to a String
                // and pull the value from the HashMap. If we fail, it's a
                let key_as_string =
                    String::from_utf8(msg.to_vec()).expect("utf8 to String conversion failed");

                // if this k,v exists, we send it back. If not, we go silent and ignore.
                if let Some(reply) = db.get(&key_as_string) {
                    info!("Retrieve message type detected, replying with {}", reply);
                    // let amt = socket.send_to(reply.as_bytes(), &peer).await?;
                    let amt = socket.send_to(&buf[..size], &peer).await?;

                    info!("Sent {} bytes back to {}.", amt, peer);
                }
            }
        }

        let amt = socket.send_to(&buf[..size], &peer).await?;
        println!("Echoed {} bytes to {}", amt, peer);
    }
}

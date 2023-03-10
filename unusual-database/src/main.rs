use std::collections::HashMap;

use tokio::net::UdpSocket;

use env_logger::Env;
use log::info;

/// All possible request types are here
enum RequestType {
    Insert,
    Retrieve,
    Version,
}

/// This function returns the key & value pair
/// if this is an INSERT op, or None otherwise
/// https://doc.rust-lang.org/std/primitive.str.html#method.split_once
fn get_kv_pair(msg: &str) -> Option<(String, String)> {
    msg.split_once('=')
        .map(|(key, value)| -> (String, String) { (key.to_string(), value.to_string()) })
}

///The function parse_request takes in a message of type &str 
/// and returns an enumeration of RequestType which can be Insert, Retrieve or Version. 
/// It checks the message for keywords "version", or "=" and returns the corresponding RequestType.
fn parse_request(msg: &str) -> RequestType {
    if msg.starts_with("version") {
        RequestType::Version
    } else if msg.contains('=') {
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

    // The main storage object for all the messages is a HashMap where the key and value are both of type String.
    let mut db: std::collections::HashMap<String, String> = HashMap::new();

    loop {
        let mut buf = vec![0u8; 1024];
        let (size, peer) = socket.recv_from(&mut buf).await?;
        let msg = String::from_utf8_lossy(&buf[..size]).to_string();
        info!("Received {} from {}", msg, peer);

        // first we figure out what type of a request this was
        match parse_request(&msg) {
            RequestType::Insert => {
                if let Some((k, v)) = get_kv_pair(&msg) {
                    info!("Insert message type detected, adding {}={}", k, v);
                    db.insert(k, v);
                }
            }
            RequestType::Retrieve => {
                // if this k,v exists, we send it back. If not, we go silent and ignore.
                if let Some(value) = db.get(&msg) {
                    // join the k,v pair backup with = separator
                    let reply = format!("{}={}",&msg,value);

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
        // let amt = socket.send_to(&buf[..size], &peer).await?;
        // println!("Echoed {} bytes to {}", amt, peer);
    }
}

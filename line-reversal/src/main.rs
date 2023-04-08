use env_logger::Env;
// use futures::{FutureExt, SinkExt, StreamExt};
use futures::StreamExt;
use line_reversal::codec::MessageCodec;
use line_reversal::message::MessageType;
use log::{error, info};
// use std::time::Duration;
use tokio::net::UdpSocket;

use tokio_util::udp::UdpFramed;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Setup the logging framework
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let socket = UdpSocket::bind("0.0.0.0:8080").await?;
    info!("Listening on {}", socket.local_addr()?);

    let mut framed = UdpFramed::new(socket, MessageCodec::new());

    while let Some(message) = framed.next().await {
        match message {
            Ok((MessageType::Connect { session }, address)) => {
                info!("Got a connect msg session {session} from {address}")
            }
            Ok((MessageType::Ack { session, length }, address)) => {
                info!("Got an ack msg session {session} length {length} from {address}")
            }

            Ok((MessageType::Data { session, pos, data }, address)) => {
                info!("Got an data msg session {session} pos {pos} data {data:?} from {address}")
            }
            Ok((MessageType::Close { session }, address)) => {
                info!("Got a close msg session {session} from {address}")
            }

            Err(e) => error!("Error: {e}"),
        }
    }
    Ok(())
}

// process(&mut socket).await?;

// async fn process(socket: &mut UdpFramed<MessageCodec>) -> Result<(), io::Error> {
//     let timeout = Duration::from_millis(200);

//     while let Ok(Some(Ok((message, addr)))) = time::timeout(timeout, socket.next()).await {
//         info!("[b] recv: {:?} from {:?}", message, addr);
//     }

//     Ok(())
// }

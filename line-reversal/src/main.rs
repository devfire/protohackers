use std::net::SocketAddr;
use std::sync::Arc;

use env_logger::Env;
// use futures::{FutureExt, SinkExt, StreamExt};
use futures::{SinkExt, StreamExt};

use line_reversal::message::MessageType;
use line_reversal::{codec::MessageCodec, state::Db};
use log::{error, info};
// use std::time::Duration;
use tokio::net::UdpSocket;

use tokio::sync::mpsc;

use tokio_util::udp::UdpFramed;

use crate::handlers::connect::handle_connect;

mod handlers;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Setup the logging framework
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let shared_db = Db::new();

    // UdpSocket does not provide a split method,
    // because this functionality can be achieved by instead wrapping the socket in an Arc.
    // Note that you do not need a Mutex to share the UdpSocket — an Arc<UdpSocket> is enough.
    // This is because all of the methods take &self instead of &mut self.
    // Once you have wrapped it in an Arc, you can call .clone() on the Arc<UdpSocket>
    // to get multiple shared handles to the same socket.
    let socket = UdpSocket::bind("0.0.0.0:8080").await?;

    info!("Listening on {}", socket.local_addr()?);
    let r = Arc::new(socket);
    let s = r.clone();
    let (tx, mut rx) = mpsc::channel::<(MessageType, SocketAddr)>(1_000);

    //
    let mut framed_read = UdpFramed::new(s, MessageCodec::new());
    let mut framed_write = UdpFramed::new(r, MessageCodec::new());

    tokio::spawn(async move {
        while let Some((msg, addr)) = rx.recv().await {
            let len = framed_write.send((msg, addr));
            info!("{:?} bytes sent", len);
        }
    });

    while let Some(message) = framed_read.next().await {
        match message {
            Ok((MessageType::Connect { session }, client_address)) => {
                info!("Got a connect msg session {session} from {client_address}");
                handle_connect(session, client_address, shared_db.clone()).await?;
                let ack_msg = MessageType::Ack { session, length: 0 };
                tx.send((ack_msg, client_address)).await?;
            }
            Ok((MessageType::Ack { session, length }, address)) => {
                info!("Got an ack msg session {session} length {length} from {address}")
            }

            Ok((MessageType::Data { session, pos_data }, address)) => {
                info!("Got a data msg session {session} from {address}")
            }

            Ok((MessageType::Close { session }, address)) => {
                info!("Got a close msg session {session} from {address}")
            }

            Err(e) => error!("Error: {e}"),
        }
    }
    Ok(())
}

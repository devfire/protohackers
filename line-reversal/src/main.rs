use env_logger::Env;
use futures::{FutureExt, SinkExt, StreamExt};
use line_reversal::codec::MessageCodec;
use log::info;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::{io, time};
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

    // loop {
        tokio::select! {
            Some(message) = framed.next() => {info!("{message:?}")},
            // else => break,

        }
    // }


    // process(&mut socket).await?;

    Ok(())
}

async fn process(socket: &mut UdpFramed<MessageCodec>) -> Result<(), io::Error> {
    let timeout = Duration::from_millis(200);

    while let Ok(Some(Ok((message, addr)))) = time::timeout(timeout, socket.next()).await {
        info!("[b] recv: {:?} from {:?}", message, addr);
    }

    Ok(())
}

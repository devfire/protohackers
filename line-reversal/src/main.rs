use env_logger::Env;
use log::info;
// use futures::{FutureExt, SinkExt};
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

    Ok(())
}

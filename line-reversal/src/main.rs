use env_logger::Env;
use std::time::Duration;

use line_reversal::codec::MessageCodec;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

use log::info;
use std::net::SocketAddrV4;
use tokio::net::UdpSocket;
use tokio_stream::StreamExt;
use tokio_util::udp::UdpFramed;

//IP constant
const IP_ANY: [u8; 4] = [0, 0, 0, 0];

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Setup the logging framework
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    //Create a udp ip4 socket
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;

    //Allow this port to be reused by other sockets
    socket.set_reuse_address(true)?;
    socket.set_nonblocking(false)?;

    //Create IPV4 any adress
    let address = SocketAddrV4::new(IP_ANY.into(), 8080);
    socket.bind(&SockAddr::from(address))?;

    //Convert to tokio udp socket
    let udp_socket = UdpSocket::from_std(socket.into())?;

    info!(
        "Created a UDP Socket at {}, {}",
        address.ip().to_string(),
        address.port().to_string()
    );

    let mut framed = UdpFramed::new(udp_socket, MessageCodec::new());

    loop {
        info!("Waiting for incoming messages");

        let result = framed.next();

        info!("{result:?}")
    }

    // Ok(())
}

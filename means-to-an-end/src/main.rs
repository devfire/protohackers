use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
fn main() -> io::Result<()>{
    println!("Hello, world!");
}

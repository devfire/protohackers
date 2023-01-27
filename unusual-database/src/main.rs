// chapter3/udp-echo-server.rs

use std::thread;
use std::net::UdpSocket;

use log::{info, error};

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:8888")
                           .expect("Could not bind socket");

    loop {
        let mut buf = [0u8; 1500];
        let sock = socket.try_clone().expect("Failed to clone socket");
        match socket.recv_from(&mut buf) {
            Ok((_, src)) => {
                thread::spawn(move || {
                    info!("Received {:?} from {}", &buf, src);
                    // sock.send_to(&buf, &src)
                    //     .expect("Failed to send a response");
                });
            },
            Err(e) => {
                error!("couldn't recieve a datagram: {}", e);
            }
        }
    }
}
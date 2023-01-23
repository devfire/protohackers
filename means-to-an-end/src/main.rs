use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};

// https://doc.rust-lang.org/std/primitive.i32.html#method.from_be_bytes
fn read_be_i32(input: &[u8]) -> i32 {
    i32::from_be_bytes(input[..4].try_into().unwrap())
}

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    loop {
        // The second item contains the ip and port of the new connection.
        let (stream, addr) = listener.accept().await.unwrap();

        println!("New connection from {}", addr);

        // A new task is spawned for each inbound socket.  The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(stream).await;
        });
    }
}

async fn process(stream: TcpStream) {
    use std::collections::HashMap;

    let (reader, mut writer) = tokio::io::split(stream);

    // A hashmap is used to store data
    let mut db = HashMap::new();

    loop {
        let mut buffer = [0; 9];
        let n = reader
            .read_exact(&mut buffer)
            .await
            .expect("Unable to read from buffer");
        // println!("Bytes received: {} buffer length: {}", n, buffer.len());
        if n == 0 {
            break;
        }

        let msg_type = &buffer[0];

        // Read slices 1,2,3,4.
        // Since we have a slice rather than an array, fallible conversion APIs can be used
        let first_half_decoded = read_be_i32(&buffer[1..=4]);
        let second_half_decoded = read_be_i32(&buffer[5..n]);

        println!(
            "Type: {}, first: {}, second: {}",
            msg_type, first_half_decoded, second_half_decoded
        );

        // Write the response to the client
        connection.write_frame(&response).await.unwrap();
    }
}

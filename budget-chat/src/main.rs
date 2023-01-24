use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    while let Ok((stream, addr)) = listener.accept().await {
        println!("New connection from {}", addr);

        // A new task is spawned for each inbound socket.  The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(stream).await;
        });
    }
}

async fn process(stream: TcpStream) {
    let (mut reader, mut writer) = tokio::io::split(stream);

    // get_username

    // notify_presence

    
}
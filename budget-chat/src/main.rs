use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

struct Users {
    username: String,
    joined: bool,
}

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

fn get_username(mut reader: io::BufReader<io::ReadHalf<TcpStream>>) -> String {
    let mut line = String::new();
    let _n = reader.read_line(&mut line);

    println!("Received: {:?}", line);
    line
}
async fn process(stream: TcpStream) {
    let (reader, mut writer) = tokio::io::split(stream);

    let reader: io::BufReader<io::ReadHalf<TcpStream>> = io::BufReader::new(reader);

    let mut users: Vec<Users>;

    let new_username = get_username(reader);

    // notify_presence
}

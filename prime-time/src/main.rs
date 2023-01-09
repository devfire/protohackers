use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt};
use tokio::net::TcpListener;
use tokio::io::BufReader;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
struct Request {
    #[validate(length(min = 1), custom = "validate_method")]
    method: String,

    #[validate(range(min = 1))]
    number: f32,
    // other fields go here
}

fn validate_method(method: &str) -> Result<(), ValidationError> {
    if method != "isPrime" {
        // the value of the username will automatically be added later
        return Err(ValidationError::new("ERROR: method must be isPrime"));
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct Response {
    // fields go here
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;

        let mut socket = BufReader::new(socket);

        tokio::spawn(async move {

            // In a loop, read data from the socket and write the data back.
            loop {

                let mut line = String::new();

                let n = match socket.read_line(&mut line).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
            }
        });
    }
}

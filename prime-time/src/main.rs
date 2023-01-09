use tokio::io::BufReader;
// use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::io::AsyncBufReadExt;
use tokio::net::TcpListener;

use serde::{Deserialize, Serialize};
// use serde_json::Value;

use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
struct Request {
    #[validate(length(min = 1), custom = "validate_method")]
    method: String,

    #[validate(range(min = 1))]
    number: f32,
}

#[derive(Debug, Serialize, Validate)]
struct Response {
    #[validate(length(min = 1), custom = "validate_method")]
    method: String,

    prime: bool,
}

fn validate_method(method: &str) -> Result<(), ValidationError> {
    if method != "isPrime" {
        return Err(ValidationError::new("ERROR: method must be isPrime"));
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;

        /*
        Wrap our socket stream in a BufReader to enhance the stream with buffering,
        which is required to use the read_line function.
        Since the JSON paylod is EOL terminated https://protohackers.com/problem/1
        we need read_line
        */
        let mut socket = BufReader::new(socket);

        tokio::spawn(async move {
            // In a loop, read data from the socket and write the data back.
            loop {
                let mut line = String::new();

                // we don't really need to keep the buffer size, only to ensure it's non 0 to proceed
                let _n = match socket.read_line(&mut line).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // attempt to deserialize the payload into JSON
                let request: Result<Request, _> = serde_json::from_str(&line);

                match request {
                    Ok(request) => {
                        let request_validation = request.validate();
                        if let Err(e) = request_validation {
                            println!("{}",e)
                        } else {
                            println!("Valid request: {:?}", request);
                        }
                    }
                    Err(e) => {
                        // request is invalid, send an error response
                        println!(
                            "ERROR: JSON parsing failed, received invalid request: {}",
                            e
                        );
                    }
                }
            }
        });
    }
}
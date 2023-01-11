use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use serde::{Deserialize, Serialize};
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
    method: String,
    prime: bool,
}

fn validate_method(method: &str) -> Result<(), ValidationError> {
    if method != "isPrime" {
        return Err(ValidationError::new("Method must be isPrime"));
    }
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    let listener = TcpListener::bind(&addr).await?;

    while let Ok((stream, addr)) = listener.accept().await {
        println!("New connection from {}", addr);

        let stream = stream;
        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = io::BufReader::new(reader);

        tokio::spawn(async move {
            loop {
                let mut line = String::new();
                let n = reader.read_line(&mut line).await.unwrap();

                // we don't really need to keep the buffer size, only to ensure it's non 0 to proceed
                match reader.read_line(&mut line).await {
                    // Ok(n) if n == 0 => return, // socket closed
                    Ok(n) => println!("Read {} bytes", n),
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // attempt to deserialize the payload into JSON
                let request: Result<Request, _> = serde_json::from_str(&line);

                match request {
                    Ok(request) => {
                        // Happy path: request is a valid JSON
                        // validate the fields in the Request struct
                        let request_validation = request.validate();
                        if let Err(e) = request_validation {
                            // Validation failed, error out
                            println!("{}", e);

                            // Convert the validation error to String and ship it back
                            writer.write_all(e.to_string().as_bytes()).await.unwrap();

                            //Flush this output stream, ensuring that all intermediately buffered contents reach their destination.
                            writer.flush().await.unwrap();

                            // return;
                        } else {
                            // Happy path: request is a valid payload
                            println!("Valid request: {:?}", request);

                            // set the prime bool to false by default
                            let mut response = Response {
                                method: String::from("isPrime"),
                                prime: false,
                            };

                            // check whether the number is prime or not
                            if primes::is_prime(request.number as u64) {
                                // flip the prime bool to true since the number is prime,
                                // otherwise it stays false
                                response.prime = true;
                            }

                            // encode the JSON response as a vec of bytes, we get back a Result<> from to_vec
                            let response_bytes = serde_json::to_vec(&response);

                            match response_bytes {
                                Ok(response_bytes) => {
                                    writer.write_all(&response_bytes).await.unwrap();
                                    
                                    //Flush this output stream, ensuring that all intermediately buffered contents reach their destination.
                                    writer.flush().await.unwrap();
                                }
                                Err(e) => {
                                    println!("ERROR: {}", e);
                                    // return;
                                }
                            };
                        }
                    }
                    Err(e) => {
                        // request is invalid JSON, send an error response
                        println!("ERROR: JSON parsing failed due to invalid request: {}", e);
                        writer.write_all("Malformed JSON".as_bytes()).await.unwrap();

                        //Flush this output stream, ensuring that all intermediately buffered contents reach their destination.
                        writer.flush().await.unwrap();
                        // return;
                    }
                }

                println!("Read {} bytes", n);
                println!("{}", line);
            }
        });
    }

    Ok(())
}

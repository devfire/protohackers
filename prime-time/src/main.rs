use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
struct Request {
    #[validate(length(min = 1), custom = "validate_method")]
    method: String,

    // #[validate(range(min = 1))]
    number: f64,
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

/// From https://docs.rs/primes/latest/src/primes/lib.rs.html
fn firstfac(x: i64) -> i64 {
    if x % 2 == 0 {
        return 2;
    };

    for n in (1..).map(|m| 2 * m + 1).take_while(|m| m * m <= x) {
        if x % n == 0 {
            return n;
        };
    }
    // No factor found. It must be prime.
    x
}

fn is_prime(n: i64) -> bool {
    if n <= 1 {
        return false;
    }
    firstfac(n) == n
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
<<<<<<< HEAD
                let n = reader
                    .read_line(&mut line)
                    .await
                    .expect("Socket read failed.");
                if n == 0 {
                    break;
                }
=======
                let n = reader.read_line(&mut line).await.unwrap();

                // we don't really need to keep the buffer size, only to ensure it's non 0 to proceed
                match reader.read_line(&mut line).await {
                    Ok(n) if n == 0 => return, // socket closed
                    Ok(n) => println!("Read {} bytes", n),
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
>>>>>>> origin/main

                println!("Received: {:?}", line);

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
                            writer
                                .write_all("Malformed request.".as_bytes())
                                .await
<<<<<<< HEAD
                                .expect("Socket write-back failed.");

                            writer.write_all(b"\n").await.unwrap();
=======
                                .unwrap();
                            return;
>>>>>>> origin/main
                        } else {
                            // Happy path: request is a valid payload
                            println!("Valid request: {:?}", request);

                            // set the prime bool to false by default
                            let mut response = Response {
                                method: String::from("isPrime"),
                                prime: false,
                            };

                            // check whether the number is prime or not.
                            // NOTE: floating point numbers are never prime.
                            if is_prime(request.number as i64) {
                                // flip the prime bool to true since the number is prime,
                                // otherwise it stays false
                                response.prime = true;
                                println!("Number {} is prime", request.number);
                            }

                            // encode the JSON response as a vec of bytes, we get back a Result<> from to_vec
                            let response_bytes = serde_json::to_vec(&response);

                            match response_bytes {
                                Ok(response_bytes) => {
                                    writer
                                        .write_all(&response_bytes)
                                        .await
                                        .expect("Socket write-back failed.");
                                    writer.write_all(b"\n").await.unwrap();
                                    println!("Sending back a response.");
                                }
                                Err(e) => {
                                    // this should never happen since we construct the response
                                    println!("ERROR: {}", e);
                                    return;
                                }
                            };
                        }
                    }
                    Err(e) => {
                        // request is invalid JSON, send an error response
<<<<<<< HEAD
                        println!("ERROR: {}", e);
                        writer
                            .write_all("Malformed JSON".as_bytes())
                            .await
                            .expect("Socket write failed.");
                        writer.write_all(b"\n").await.unwrap();
=======
                        println!("ERROR: JSON parsing failed due to invalid request: {}", e);
                        writer.write_all("Malformed JSON".as_bytes()).await.unwrap();
                        return;
>>>>>>> origin/main
                    }
                }

                // println!("Read {} bytes", n);
                // println!("{}", line);

                // Echo the data back to the client
            }
        });
    }

    Ok(())
}

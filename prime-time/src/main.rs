use tokio::net::TcpListener;
use tokio::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    // fields go here
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    // fields go here
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:12345".parse()?;
    let listener = TcpListener::bind(&addr).await?;

    let server = async move {
        let mut incoming = listener.incoming();
        while let Some(conn) = incoming.next().await {
            let conn = conn?;
            tokio::spawn(async move {
                let mut buf = Vec::new();
                // read from the connection and append the data to `buf`
                let n = conn.read_buf(&mut buf).await?;
                // try to deserialize the request
                let request: Result<Request, _> = serde_json::from_slice(&buf[..n]);
                match request {
                    Ok(request) => {
                        // request is valid, handle it
                        println!("received request: {:?}", request);
                        let response = Response { /* fields go here */ };
                        let response_bytes = serde_json::to_vec(&response)?;
                        conn.write_all(&response_bytes).await?;
                    }
                    Err(e) => {
                        // request is invalid, send an error response
                        println!("received invalid request: {}", e);
                        let response = Response { /* fields go here */ };
                        let response_bytes = serde_json::to_vec(&response)?;
                        conn.write_all(&response_bytes).await?;
                    }
                }
                Ok(())
            });
        }
    };

    server.await?;

    Ok(())
}

// Completed till 3.2

use crate::resp::{RESP, bytes_to_resp};
use crate::server::process_request;
use crate::storage::Storage;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

mod resp;
mod resp_result;
mod server;
mod storage;
mod storage_result;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Create the TCP listener, bound to the
    // standard Redis port.
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    // Create a storage and protect it against concurrency issues.
    let storage = Arc::new(Mutex::new(Storage::new()));

    let mut interval_timer = tokio::time::interval(Duration::from_millis(10));

    loop {
        tokio::select! {
            connection = listener.accept() => {
                match connection {
                    Ok((stream, _)) => {
                        tokio::spawn(handle_connection(stream, storage.clone()));
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        continue;
                    }
                }
            }

            _ = interval_timer.tick() => {
                    tokio::spawn(expire_keys(storage.clone()));
                }
        }
    }
}

// The main entry point for valid TCP connections
async fn handle_connection(mut stream: TcpStream, storage: Arc<Mutex<Storage>>) {
    // Create a buffer to host incoming data.
    let mut buffer = [0; 512];

    loop {
        // Read from the stream into the buffer
        match stream.read(&mut buffer).await {
            // If th stream returned some data,
            // process the request.
            Ok(size) if size != 0 => {
                // Hardcoded response using a specific variant
                let mut index: usize = 0;

                let request = match bytes_to_resp(&buffer[..size].to_vec(), &mut index) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return;
                    }
                };

                let response = match process_request(request, storage.clone()) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error parsing command: {}", e);
                        return;
                    }
                };

                if let Err(e) = stream.write_all(response.to_string().as_bytes()).await {
                    eprintln!("Error writing to socket: {}", e);
                }
            }
            // If the stream returned no data
            // the connection has been closed
            Ok(_) => {
                println!("Connection closed");
                break;
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }
}

async fn expire_keys(storage: Arc<Mutex<Storage>>) {
    let mut guard = storage.lock().unwrap();

    guard.expire_keys();
}

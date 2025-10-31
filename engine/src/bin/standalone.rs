use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

type ClientMap = Arc<Mutex<HashMap<String, TcpStreamHandle>>>;

#[derive(Clone)]
struct TcpStreamHandle {
    writer: tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>,
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Technomancy server listening on 127.0.0.1:8080");

    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let addr_str = addr.to_string();
        println!("New client connected: {}", addr_str);

        let clients_clone = clients.clone();
        let (reader, writer) = socket.into_split();
        let handle = TcpStreamHandle {
            writer: tokio::sync::Mutex::new(writer),
        };

        clients_clone.lock().unwrap().insert(addr_str.clone(), handle.clone());

        tokio::spawn(handle_client(addr_str, reader, clients_clone));
    }
}

async fn handle_client(addr: String, mut reader: tokio::net::tcp::OwnedReadHalf, clients: ClientMap) {
    let mut buf = [0u8; 1024];

    loop {
        match reader.read(&mut buf).await {
            Ok(0) => break, // disconnected
            Ok(n) => {
                let input = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                println!("{} says: {}", addr, input);

                // Broadcast message to all clients
                let clients_locked = clients.lock().unwrap();
                for (other_addr, client) in clients_locked.iter() {
                    if other_addr != &addr {
                        let mut writer = client.writer.lock().await;
                        let _ = writer.write_all(format!("{} says: {}\n", addr, input).as_bytes()).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from {}: {}", addr, e);
                break;
            }
        }
    }

    println!("Client disconnected: {}", addr);
    clients.lock().unwrap().remove(&addr);
}

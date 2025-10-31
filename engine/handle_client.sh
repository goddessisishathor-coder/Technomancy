async fn handle_client(addr: String, mut reader: tokio::net::tcp::OwnedReadHalf, clients: ClientMap) {
    let mut buf = [0u8; 1024];

    loop {
        match reader.read(&mut buf).await {
            Ok(0) => break, // disconnected
            Ok(n) => {
                let input = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                println!("{} says: {}", addr, input);

                // Parse command
                let response = if input == "ping" {
                    "pong\n".to_string()
                } else if input.starts_with("cast ") {
                    let spell = input.strip_prefix("cast ").unwrap();
                    format!("You cast {}!\n", spell)
                } else if input == "look" {
                    "You see a mysterious digital realm.\n".to_string()
                } else {
                    format!("Unknown command: {}\n", input)
                };

                // Send response back to this client
                let clients_locked = clients.lock().unwrap();
                if let Some(client) = clients_locked.get(&addr) {
                    let mut writer = client.writer.lock().await;
                    let _ = writer.write_all(response.as_bytes()).await;
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

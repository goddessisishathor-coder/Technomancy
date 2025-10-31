use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server listening on 127.0.0.1:8080");

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("New client connected: {}", addr);

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => break, // Connection closed
                    Ok(n) => {
                        let input = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                        println!("{} says: {}", addr, input);

                        // Simple command handling
                        let response = match input.as_str() {
                            "ping" => "pong\n".to_string(),
                            "hello" => "Greetings, human!\n".to_string(),
                            _ => format!("Unknown command: {}\n", input),
                        };

                        if let Err(e) = socket.write_all(response.as_bytes()).await {
                            eprintln!("Failed to write to socket: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read from socket: {}", e);
                        break;
                    }
                }
            }

            println!("Client disconnected: {}", addr);
        });
    }
}

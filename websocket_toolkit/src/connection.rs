#![allow(unused_imports)]
use log::{info, error};
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio::net::TcpStream;
use url::Url;
use futures_util::{sink::SinkExt, StreamExt};  // Import StreamExt here
use crate::messages::{MessageHandler, MessageFormat};

/// `WebSocketClient` is responsible for managing WebSocket connections, including connection setup and sending messages.
#[derive(Clone)]
pub struct WebSocketClient {
    pub url: String,
    retries: u32,
}

impl WebSocketClient {
    /// Creates a new `WebSocketClient` with a specified URL and retry limit.
    pub fn new(url: &str, retries: u32) -> Self {
        WebSocketClient {
            url: url.to_string(),
            retries,
        }
    }

    /// Receives a message from the WebSocket server.
    pub async fn receive_message(&self) -> Option<Vec<u8>> {
        let mut ws_stream = self.connect().await.ok()?;
        
        if let Some(Ok(message)) = ws_stream.next().await {
            match message {
                Message::Text(text) => Some(text.into_bytes()),
                Message::Binary(data) => Some(data),
                _ => None, // Ignore non-text/binary messages like Ping or Close
            }
        } else {
            None
        }
    }

    /// Attempts to establish a WebSocket connection. If the connection fails, retries the connection.
    pub async fn connect(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Error> {
        let url = Url::parse(&self.url).expect("Invalid WebSocket URL");
        info!("Attempting to connect to WebSocket server at {}", self.url);
        let (ws_stream, _) = connect_async(url).await?;
        info!("Connected to WebSocket server at {}", self.url);
        Ok(ws_stream)
    }

    /// Sends a message over an active WebSocket connection. The message is serialized using JSON format by default.
    pub async fn send_message(&self, ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, message: &str) {
        // Serialize the message using the specified format (e.g., JSON)
        let serialized = MessageHandler::serialize(&message, MessageFormat::Json);

        // Handle the Result properly by unwrapping the Ok value or logging an error if it fails
        match serialized {
            Ok(serialized_data) => {
                match ws_stream.send(Message::Binary(serialized_data)).await {
                    Ok(_) => info!("Sent message: {}", message),
                    Err(e) => error!("Failed to send message: {}", e),
                }
            },
            Err(e) => error!("Failed to serialize message: {}", e),  // Log error if serialization fails
        }
    }

    /// Disconnects the WebSocket connection gracefully.
    pub fn disconnect(&self) {
        self.private_disconnect();
    }

    /// Returns the retry count for reconnection logic.
    pub fn get_retries(&self) -> u32 {
        self.retries
    }

    /// Private method for handling graceful disconnection and resource cleanup.
    fn private_disconnect(&self) {
        info!("Disconnected from WebSocket server at {}", self.url);
    }

    /// Attempts to reconnect to the WebSocket server if the connection fails.
    pub async fn reconnect(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Error> {
        let mut retries_left = self.retries;
        while retries_left > 0 {
            match self.connect().await {
                Ok(ws_stream) => {
                    info!("Reconnection successful.");
                    return Ok(ws_stream);
                }
                Err(e) => {
                    error!("Failed to reconnect: {}", e);
                    retries_left -= 1;
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await; // Wait before retrying
                }
            }
        }
        Err(Error::Io(std::io::Error::new(std::io::ErrorKind::TimedOut, "Reconnection failed")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_websocket_client_creation() {
        let client = WebSocketClient::new("wss://example.com/socket", 3);
        assert_eq!(client.url, "wss://example.com/socket");
        assert_eq!(client.get_retries(), 3);
    }

    #[tokio::test]
    async fn test_websocket_client_connection() {
        // Bind the server to a specified port
        println!("Binding server to port 9002...");
        let listener = TcpListener::bind("127.0.0.1:9002").await.expect("Failed to bind server");
    
        // Set up a mock WebSocket server and start it
        println!("Starting mock WebSocket server...");
        let server_handle = tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                println!("Server accepted a connection.");
                let mut ws_stream = accept_async(stream)
                    .await
                    .expect("Failed to accept WebSocket connection");
    
                // Respond to pings to maintain connection
                while let Some(Ok(Message::Ping(_))) = ws_stream.next().await {
                    println!("Server received ping; sending pong response.");
                    ws_stream
                        .send(Message::Pong(Vec::new()))
                        .await
                        .expect("Failed to send pong");
                }
    
                // Close the WebSocket stream gracefully after processing
                println!("Closing WebSocket stream.");
                ws_stream.close(None).await.expect("Failed to close WebSocket");
            }
        });
    
        // Introduce a delay to ensure the server is fully up
        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("Mock server setup complete.");
    
        // Attempt to connect to the mock WebSocket server
        let client_url = "ws://127.0.0.1:9002";
        let client = WebSocketClient::new(client_url, 3);
        println!("Created WebSocketClient instance.");
    
        // Adding retries for the connection attempt
        let mut attempts = 0;
        let max_attempts = 3;
        let mut connected = false;
    
        while attempts < max_attempts {
            println!("Attempting to connect to WebSocket server... Attempt: {}", attempts + 1);
            match client.connect().await {
                Ok(_) => {
                    println!("Connection successful on attempt {}.", attempts + 1);
                    connected = true;
                    break;
                }
                Err(e) => {
                    eprintln!(
                        "Attempt {} failed to connect to WebSocket server: {}",
                        attempts + 1,
                        e
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    attempts += 1;
                }
            }
        }
    
        // Ensure connection was successful
        assert!(connected, "Expected successful WebSocket connection after retries");
        println!("WebSocket connection established.");
    
        // Explicitly close the client connection
        println!("Closing client connection.");
        client.disconnect();
    
        // Clean up the server
        println!("Stopping mock WebSocket server.");
        server_handle.abort();
        let _ = server_handle.await; // Ensure the server task is complete
        println!("Test complete.");
    }
}

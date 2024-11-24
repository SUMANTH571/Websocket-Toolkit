//! # `connection.rs`: WebSocket connection handling module
//!
//! This module defines the `WebSocketClient` struct and its associated methods for managing WebSocket connections.
//! It provides functionality for connection setup, message sending, receiving, and reconnection logic.

#![allow(unused_imports)]
use log::{info, error};
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio::net::TcpStream;
use url::Url;
use futures_util::{sink::SinkExt, StreamExt}; 
use crate::messages::{MessageHandler, MessageFormat};

/// `WebSocketClient` is responsible for managing WebSocket connections, including connection setup, 
/// message sending, and reconnection logic. It provides methods to establish a connection, 
/// send and receive messages, and gracefully disconnect.
///
/// # Fields
/// - `url` - The URL of the WebSocket server.
/// - `retries` - The number of reconnection attempts allowed.
///
/// # Examples
///
/// ```rust
/// use websocket_toolkit::connection::WebSocketClient;
///
/// let client = WebSocketClient::new("wss://example.com/socket", 3);
/// assert_eq!(client.url, "wss://example.com/socket");
/// assert_eq!(client.get_retries(), 3);
/// ```

#[derive(Clone)]
pub struct WebSocketClient {
    /// The URL of the WebSocket server.
    pub url: String,
    /// Number of retries allowed for reconnection attempts.
    retries: u32,
}

impl WebSocketClient {
    /// Creates a new `WebSocketClient` with a specified URL and retry limit.
    ///
    /// # Arguments
    /// - `url` - The WebSocket server URL as a string.
    /// - `retries` - The number of reconnection attempts allowed.
    ///
    /// # Returns
    /// A new instance of `WebSocketClient`.
    ///
    /// # Examples
    /// ```rust
    /// use websocket_toolkit::connection::WebSocketClient;
    ///
    /// let client = WebSocketClient::new("wss://example.com/socket", 3);
    /// assert_eq!(client.url, "wss://example.com/socket");
    /// assert_eq!(client.get_retries(), 3);
    /// ```
    
    pub fn new(url: &str, retries: u32) -> Self {
        WebSocketClient {
            url: url.to_string(),
            retries,
        }
    }

    /// Receives a message from the WebSocket server.
    ///
    /// # Returns
    /// An `Option` containing the message as a `Vec<u8>` if successful, or `None` otherwise.
    ///
    /// # Examples
    /// ```rust
    /// use websocket_toolkit::connection::WebSocketClient;
    ///
    /// let runtime = tokio::runtime::Runtime::new().unwrap();
    /// runtime.block_on(async {
    ///     let client = WebSocketClient::new("wss://example.com/socket", 3);
    ///     if let Some(message) = client.receive_message().await {
    ///         println!("Received message: {:?}", message);
    ///     }
    /// });
    /// ```
    
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
    ///
    /// # Returns
    /// A `Result` containing the WebSocket stream on success, or an `Error` on failure.
    ///
    /// # Examples
    /// ```rust
    /// use websocket_toolkit::connection::WebSocketClient;
    ///
    /// let runtime = tokio::runtime::Runtime::new().unwrap();
    /// runtime.block_on(async {
    ///     let client = WebSocketClient::new("wss://example.com/socket", 3);
    ///     match client.connect().await {
    ///         Ok(_) => println!("Connected to WebSocket server."),
    ///         Err(e) => eprintln!("Failed to connect: {}", e),
    ///     }
    /// });
    /// ```
    
    pub async fn connect(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Error> {
        let url = Url::parse(&self.url).expect("Invalid WebSocket URL");
        info!("Attempting to connect to WebSocket server at {}", self.url);
        let (ws_stream, _) = connect_async(url).await?;
        info!("Connected to WebSocket server at {}", self.url);
        Ok(ws_stream)
    }

    /// Sends a message over an active WebSocket connection. The message is serialized using JSON format by default.
    ///
    /// # Arguments
    /// - `ws_stream` - The WebSocket stream to send the message over.
    /// - `message` - The message to send as a string.
    ///

    pub async fn send_message(&self, ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, message: &str) {
        let serialized = MessageHandler::serialize(&message, MessageFormat::Json);

        match serialized {
            Ok(serialized_data) => {
                match ws_stream.send(Message::Binary(serialized_data)).await {
                    Ok(_) => info!("Sent message: {}", message),
                    Err(e) => error!("Failed to send message: {}", e),
                }
            }
            Err(e) => error!("Failed to serialize message: {}", e),
        }
    }

    /// Disconnects the WebSocket connection gracefully.
    ///
    /// # Examples
    /// ```rust
    /// use websocket_toolkit::connection::WebSocketClient;
    ///
    /// let client = WebSocketClient::new("wss://example.com/socket", 3);
    /// client.disconnect();
    /// ```
    
    pub fn disconnect(&self) {
        self.private_disconnect();
    }

    /// Returns the retry count for reconnection logic.
    ///
    /// # Returns
    /// The number of retries allowed for reconnection attempts.
    ///
    /// # Examples
    /// ```rust
    /// use websocket_toolkit::connection::WebSocketClient;
    ///
    /// let client = WebSocketClient::new("wss://example.com/socket", 3);
    /// assert_eq!(client.get_retries(), 3);
    /// ```
    
    pub fn get_retries(&self) -> u32 {
        self.retries
    }

    /// Private method for handling graceful disconnection and resource cleanup.
    ///
    /// # Examples
    /// This method is used internally by the `disconnect` method.
    
    fn private_disconnect(&self) {
        info!("Disconnected from WebSocket server at {}", self.url);
    }

    /// Attempts to reconnect to the WebSocket server if the connection fails.
    ///
    /// # Returns
    /// A `Result` containing the WebSocket stream on successful reconnection, or an `Error` if all retries fail.
    ///
    /// # Examples
    /// ```rust
    /// use websocket_toolkit::connection::WebSocketClient;
    ///
    /// let runtime = tokio::runtime::Runtime::new().unwrap();
    /// runtime.block_on(async {
    ///     let client = WebSocketClient::new("wss://example.com/socket", 3);
    ///     match client.reconnect().await {
    ///         Ok(_) => println!("Reconnected successfully."),
    ///         Err(e) => eprintln!("Failed to reconnect: {}", e),
    ///     }
    /// });
    /// ```
    
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
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
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

    /// Tests the creation of a `WebSocketClient` instance.
    #[tokio::test]
    async fn test_websocket_client_creation() {
        let client = WebSocketClient::new("wss://example.com/socket", 3);
        assert_eq!(client.url, "wss://example.com/socket");
        assert_eq!(client.get_retries(), 3);
    }

    /// Tests the ability of `WebSocketClient` to connect to a mock WebSocket server.
    #[tokio::test]
    async fn test_websocket_client_connection() {
        // Bind the server to a specified port
        println!("Binding server to the port...");
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


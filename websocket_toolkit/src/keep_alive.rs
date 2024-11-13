use tokio::time::{interval, Duration};
use log::{info, error};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio::net::TcpStream;
use futures_util::sink::SinkExt;

pub struct KeepAlive {
    ping_interval: Duration,
}

impl KeepAlive {
    pub fn new(ping_interval: Duration) -> Self {
        KeepAlive { ping_interval }
    }

    /// Starts sending pings to keep the WebSocket connection alive.
    ///
    /// # Arguments
    ///
    /// * `ws_stream` - The WebSocket stream to send ping messages.
    ///
    /// # Returns
    ///
    /// `Result<(), String>` - Returns an error message on failure.
    pub async fn start(&self, ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> Result<(), String> {
        let mut interval = interval(self.ping_interval);

        loop {
            interval.tick().await;

            match ws_stream.send(Message::Ping(vec![])).await {
                Ok(_) => info!("Ping sent to keep connection alive"),
                Err(e) => {
                    error!("Failed to send ping: {}", e);
                    return Err(format!("Failed to send ping: {}", e)); // Return detailed error message
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio_tungstenite::{accept_async, tungstenite::Message};
    use tokio::time::{timeout, Duration};
    use futures_util::StreamExt;
    use crate::connection::WebSocketClient;
    use tokio::sync::oneshot;
    use tokio::select;

    #[tokio::test]
    async fn test_keep_alive_creation() {
        let keep_alive = KeepAlive::new(Duration::from_secs(10));
        assert_eq!(keep_alive.ping_interval, Duration::from_secs(10));
    }

    // #[ignore]
    // #[tokio::test]
async fn test_keep_alive_with_mock_server() {
    // Set up a oneshot channel to notify when the server is ready
    let (server_ready_tx, server_ready_rx) = oneshot::channel::<()>();
    
    // Set up a mock WebSocket server
    let listener = TcpListener::bind("127.0.0.1:9001")
        .await
        .expect("Failed to bind server");
    
    println!("Mock WebSocket server is being set up on 127.0.0.1:9001");

    // Spawn a task to run the server in the background
    let server_handle = tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            println!("Server accepted connection from client.");
            let mut ws_stream = accept_async(stream)
                .await
                .expect("Failed to accept WebSocket connection");

            // Notify that the server is ready
            let _ = server_ready_tx.send(());
            println!("Server is ready to accept connections.");

            // Listen for ping messages from the client and respond with pong
            while let Some(Ok(Message::Ping(_))) = ws_stream.next().await {
                println!("Received ping from client, sending pong...");
                ws_stream
                    .send(Message::Pong(Vec::new())) // Respond with Pong
                    .await
                    .expect("Failed to send pong response");
            }
        }
    });

    // Wait for the server to be ready
    match server_ready_rx.await {
        Ok(_) => println!("Server is ready to accept connections."),
        Err(_) => panic!("Failed to receive server readiness signal."),
    }

    // Set up the client side (KeepAlive) to connect to the mock server
    let client_url = "ws://127.0.0.1:9001";
    let client = WebSocketClient::new(client_url, 3);

    println!("Attempting to connect to WebSocket server at {}", client_url);
    
    // Attempt to establish the WebSocket connection
    let mut ws_stream = match client.connect().await {
        Ok(stream) => {
            println!("Connection established successfully with server.");
            stream
        }
        Err(e) => {
            println!("Failed to connect to WebSocket server: {:?}", e);
            panic!("Connection failed");
        }
    };

    // Create KeepAlive with a short interval for testing
    let keep_alive = KeepAlive::new(Duration::from_secs(1));
    println!("Starting keep-alive mechanism with a 1-second interval.");

    // Run the keep-alive ping mechanism with a timeout to avoid infinite loops in tests
    let keep_alive_handle = tokio::spawn(async move {
        keep_alive.start(&mut ws_stream).await.unwrap_or_else(|e| {
            error!("Keep-alive error: {}", e); // Log error if keep-alive fails
        });
    });

    // Set a reasonable timeout for the keep-alive mechanism and handle potential errors
    match timeout(Duration::from_secs(10), keep_alive_handle).await {
        Ok(_) => println!("Keep-alive test completed successfully."),
        Err(e) => {
            println!("Keep-alive test timed out: {:?}", e);
            panic!("Keep-alive test failed due to timeout");
        }
    }

    // Cleanup the server handle after the test
    server_handle.abort();
    let _ = server_handle.await;

    println!("Test completed and server cleaned up.");
}




    // #[tokio::test]
    // async fn test_keep_alive_with_mock_server() {
    //     // Set up a mock WebSocket server
    //     let listener = TcpListener::bind("127.0.0.1:9001").await.expect("Failed to bind server");

    //     // Spawn a task to run the server in the background
    //     let server_handle = tokio::spawn(async move {
    //         if let Ok((stream, _)) = listener.accept().await {
    //             let mut ws_stream = accept_async(stream).await.expect("Failed to accept WebSocket connection");

    //             // Listen for ping messages from the client
    //             while let Some(Ok(Message::Ping(_))) = ws_stream.next().await {
    //                 println!("Received ping from client");
    //             }
    //         }
    //     });

    //     // Set up the client side (KeepAlive) to connect to the mock server
    //     let client_url = "ws://127.0.0.1:9001";
    //     let client = WebSocketClient::new(client_url, 3);
    //     let mut ws_stream = client.connect().await.expect("Failed to connect to mock WebSocket server");

    //     // Create KeepAlive with a short interval for testing
    //     let keep_alive = KeepAlive::new(Duration::from_secs(1));
        
    //     // Run the keep-alive ping mechanism with a timeout to avoid infinite loops in tests
    //     let keep_alive_handle = tokio::spawn(async move {
    //         keep_alive.start(&mut ws_stream).await.unwrap_or_else(|e| {
    //             error!("Keep-alive error: {}", e); // Log error if keep-alive fails
    //         });
    //     });

    //     // Set a reasonable timeout for the keep-alive mechanism and handle potential errors
    //     match timeout(Duration::from_secs(5), keep_alive_handle).await {
    //         Ok(_) => println!("Keep-alive test completed successfully."),
    //         Err(e) => println!("Keep-alive test timed out: {:?}", e),
    //     }

    //     // Clean up the server handle after the test
    //     server_handle.abort();
    //     let _ = server_handle.await;
    // }
}

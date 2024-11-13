#![allow(unused_imports)]
pub mod connection;
pub mod reconnection;
pub mod messages;
pub mod keep_alive;
pub mod controller;

use futures_util::{StreamExt, SinkExt};  // Ensure these imports are included
use crate::reconnection::Connectable;
use tokio_tungstenite::tungstenite::protocol::Message;

// MockWebSocketClient to simulate a failure for testing
pub struct MockWebSocketClient;

#[async_trait::async_trait]
impl Connectable for MockWebSocketClient {
    async fn connect(&self) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        // Simulate a connection failure
        Err(tokio_tungstenite::tungstenite::Error::ConnectionClosed)
    }
}

#[cfg(test)]
mod tests {
    use super::connection::WebSocketClient;
    use super::messages::{MessageHandler, MessageFormat};
    use super::reconnection::ReconnectStrategy;
    use super::controller::WebSocketController;
    use super::MockWebSocketClient;
    use tokio::net::TcpListener;
    use tokio::time::Duration;
    use log::{error};
    use tokio_tungstenite::accept_async;
    use tokio_tungstenite::tungstenite::protocol::Message;
    use std::sync::Arc;
    use futures_util::{StreamExt, SinkExt}; 

    #[tokio::test]
    async fn test_websocket_client_connection() {
        // Bind to port 0 to get an available port
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind server");

        // Get the assigned port and use it to construct the WebSocket URL
        let local_addr = listener.local_addr().expect("Failed to get local address");
        let client_url = format!("ws://{}", local_addr);

        // Set up the mock server
        let server_handle = tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let mut ws_stream = accept_async(stream)
                    .await
                    .expect("Failed to accept WebSocket connection");

                // Listen for ping messages from the client
                while let Some(Ok(Message::Ping(_))) = ws_stream.next().await {
                    ws_stream
                        .send(Message::Pong(Vec::new()))
                        .await
                        .expect("Failed to send pong");
                }
            }
        });

        tokio::time::sleep(Duration::from_secs(1)).await;

        let client = WebSocketClient::new(&client_url, 3);

        let mut attempts = 0;
        let max_attempts = 3;
        let mut connected = false;

        // Try connecting up to `max_attempts`
        while attempts < max_attempts {
            match client.connect().await {
                Ok(_) => {
                    connected = true;
                    break;
                }
                Err(e) => {
                    error!("Attempt {} failed to connect to WebSocket server: {}", attempts + 1, e);
                    tokio::time::sleep(Duration::from_secs(4)).await;
                    attempts += 1;
                }
            }
        }

        assert!(connected, "Expected successful WebSocket connection after retries");

        // Cleanup after the test
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_reconnect_strategy_with_backoff() {
        let reconnect_strategy = ReconnectStrategy::new(3, 1);
        let client = Arc::new(MockWebSocketClient);

        // Attempt reconnection with backoff
        let reconnect_result = reconnect_strategy.reconnect(client).await;
        assert!(reconnect_result.is_none(), "Expected reconnection to stop after max retries");
    }

    #[tokio::test]
    async fn test_websocket_controller_lifecycle() {
        // Create a WebSocketController instance and test its lifecycle methods
        let mut controller = WebSocketController::new("wss://example.com/socket", 3, Some(5));

        let connect_result = controller.connect_and_send_message(b"Hello, WebSocket!").await;
        assert!(connect_result.is_err(), "Expected connection to fail with mock setup");

        controller.reconnect_if_needed().await;
        controller.maintain_connection().await;
    }

    #[test]
    fn test_message_serialization_and_deserialization() {
        let message = "Hello, WebSocket!" ;

        // Test JSON serialization and deserialization
        let serialized_json = MessageHandler::serialize(&message, MessageFormat::Json).unwrap();
        assert!(!serialized_json.is_empty(), "Expected non-empty serialized JSON data");

        let deserialized_json: Option<String> =
            MessageHandler::deserialize(&serialized_json, MessageFormat::Json).expect("Failed to deserialize JSON");
        assert_eq!(deserialized_json, Some(message.to_string()), "Expected deserialized JSON to match original message");

        // Test CBOR serialization and deserialization
        let serialized_cbor = MessageHandler::serialize(&message, MessageFormat::Cbor).unwrap();
        assert!(!serialized_cbor.is_empty(), "Expected non-empty serialized CBOR data");

        let deserialized_cbor: Option<String> =
            MessageHandler::deserialize(&serialized_cbor, MessageFormat::Cbor).expect("Failed to deserialize CBOR");
        assert_eq!(deserialized_cbor, Some(message.to_string()), "Expected deserialized CBOR to match original message");
    }
}

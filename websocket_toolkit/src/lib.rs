#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

/// Module for WebSocket connection handling.
///
/// This module contains functionality to manage WebSocket connections,
/// including connection establishment, message sending, and graceful disconnection.
pub mod connection;

/// Module for reconnection strategies.
///
/// This module defines strategies for handling reconnection attempts
/// with retry logic and exponential backoff mechanisms.
pub mod reconnection;

/// Module for message handling, including serialization and deserialization.
///
/// This module supports handling messages in different formats, such as JSON
/// and CBOR, for serialization and deserialization operations.
pub mod messages;

/// Module for WebSocket keep-alive mechanisms.
///
/// This module provides a mechanism to maintain active WebSocket connections
/// by sending periodic pings to the server to prevent timeouts.
pub mod keep_alive;

/// Module for WebSocket controller logic, managing connections and communication.
///
/// This module defines a controller that centralizes WebSocket connection
/// management, message handling, and reconnection strategies.
pub mod controller;

use crate::reconnection::Connectable;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::Mutex;

/// A mock WebSocket client for testing purposes.
///
/// This struct simulates a WebSocket client that always fails to connect,
/// which is useful for testing reconnection logic and error handling.
pub struct MockWebSocketClient;

#[async_trait::async_trait]
impl Connectable for MockWebSocketClient {
    /// Simulates a connection failure for the mock WebSocket client.
    ///
    /// This method always returns a `ConnectionClosed` error, simulating
    /// a scenario where the WebSocket client cannot establish a connection.
    async fn connect(&self) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        Err(tokio_tungstenite::tungstenite::Error::ConnectionClosed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::connection::WebSocketClient;
    use super::messages::{MessageHandler, MessageFormat};
    use super::reconnection::ReconnectStrategy;
    use super::controller::WebSocketController;
    use tokio::net::TcpListener;
    use tokio::time::Duration;
    use log::error;
    use tokio_tungstenite::accept_async;
    use tokio_tungstenite::tungstenite::protocol::Message;
    use std::sync::Arc;
    use futures_util::{StreamExt, SinkExt};

    /// Tests the ability of `WebSocketClient` to establish a connection with a mock server.
    ///
    /// This test sets up a mock WebSocket server, attempts to connect using the client,
    /// and verifies successful connection after retries.
    #[tokio::test]
    async fn test_websocket_client_connection() {
        // Bind to an available port.
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind server");

        let local_addr = listener.local_addr().expect("Failed to get local address");
        let client_url = format!("ws://{}", local_addr);

        // Set up the mock server.
        let server_handle = tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let mut ws_stream = accept_async(stream)
                    .await
                    .expect("Failed to accept WebSocket connection");

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

        // Cleanup after the test.
        server_handle.abort();
    }

    /// Tests the reconnection strategy using exponential backoff.
    ///
    /// This test verifies that the reconnection strategy stops after the
    /// maximum number of retries if the connection cannot be re-established.
    #[tokio::test]
    async fn test_reconnect_strategy_with_backoff() {
        let reconnect_strategy = ReconnectStrategy::new(3, 1);
        let client = Arc::new(MockWebSocketClient);

        let reconnect_result = reconnect_strategy.reconnect(client).await;
        assert!(reconnect_result.is_none(), "Expected reconnection to stop after max retries");
    }

    /// Tests the full lifecycle of a WebSocket controller.
    ///
    /// This test verifies the controller's ability to manage WebSocket connections,
    /// including initial connection, reconnection, and keep-alive mechanisms.
    #[tokio::test]
    async fn test_websocket_controller_lifecycle() {
        let mut controller = WebSocketController::new("ws://node_server:9001", 3, Some(5));
        let connect_result = controller.connect_and_send_message(b"Hello, WebSocket!").await;
        assert!(
            connect_result.is_ok(),
            "Expected connection to succeed, but it failed: {:?}",
            connect_result.err()
        );

        let reconnect_result = controller.reconnect_if_needed().await;
        assert!(
            reconnect_result.is_ok(),
            "Reconnection logic failed: {:?}",
            reconnect_result.err()
        );

        let ws_stream = Arc::new(Mutex::new(
            controller.connect().await.expect("Failed to connect to WebSocket server"),
        ));

        let maintain_result = controller.maintain_connection(ws_stream.clone()).await;
        assert!(
            maintain_result.is_ok(),
            "Failed to maintain WebSocket connection: {:?}",
            maintain_result.err()
        );
    }

    /// Tests serialization and deserialization of messages in JSON and CBOR formats.
    ///
    /// This test ensures that messages are correctly serialized into both JSON and CBOR
    /// formats and can be deserialized back into their original structure.
    #[test]
    fn test_message_serialization_and_deserialization() {
        let message = "Hello, WebSocket!";

        let serialized_json = MessageHandler::serialize(&message, MessageFormat::Json).unwrap();
        assert!(!serialized_json.is_empty(), "Expected non-empty serialized JSON data");

        let deserialized_json: Option<String> =
            MessageHandler::deserialize(&serialized_json, MessageFormat::Json).expect("Failed to deserialize JSON");
        assert_eq!(deserialized_json, Some(message.to_string()), "Expected deserialized JSON to match original message");

        let serialized_cbor = MessageHandler::serialize(&message, MessageFormat::Cbor).unwrap();
        assert!(!serialized_cbor.is_empty(), "Expected non-empty serialized CBOR data");

        let deserialized_cbor: Option<String> =
            MessageHandler::deserialize(&serialized_cbor, MessageFormat::Cbor).expect("Failed to deserialize CBOR");
        assert_eq!(deserialized_cbor, Some(message.to_string()), "Expected deserialized CBOR to match original message");
    }
}

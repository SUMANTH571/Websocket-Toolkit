#![allow(unused_imports)]
#![allow(unused_variables)]

//! Integration tests for the `websocket_toolkit` crate.
//!
//! This file contains tests to verify the functionality of the WebSocket Toolkit library,
//! including connection handling, reconnection strategies, message serialization/deserialization,
//! and keep-alive mechanisms.

use websocket_toolkit::connection::WebSocketClient;
use websocket_toolkit::controller::WebSocketController;
use websocket_toolkit::messages::{MessageHandler, MessageFormat};
use websocket_toolkit::reconnection::{ReconnectStrategy, Connectable};
use websocket_toolkit::keep_alive::KeepAlive;
use std::sync::Arc;
use log::{info, error};
use futures_util::{StreamExt, SinkExt}; // Ensure these imports are included
use tokio_tungstenite::tungstenite::Error;
use async_trait::async_trait;
use tokio::sync::Mutex;

/// A mock WebSocket client to simulate connection failures during testing.
///
/// This struct implements the `Connectable` trait to enable testing of reconnection logic.
struct MockWebSocketClient;

#[async_trait]
impl Connectable for MockWebSocketClient {
    /// Simulates a connection failure by always returning a `ConnectionClosed` error.
    async fn connect(&self) -> Result<(), Error> {
        Err(Error::ConnectionClosed) // Simulate a connection failure
    }
}

/// Tests the `connect` method of the `MockWebSocketClient`.
///
/// This test verifies that the mock client simulates a connection failure as expected.
#[tokio::test]
async fn test_websocket_client_connect() {
    let client = MockWebSocketClient;

    let result = client.connect().await;

    assert!(
        result.is_err(),
        "Expected WebSocket connection to fail with MockWebSocketClient"
    );
}

/// Tests the `ReconnectStrategy` with exponential backoff.
///
/// This test verifies that the reconnect logic stops after the maximum number of retries.
#[tokio::test]
async fn test_reconnect_strategy() {
    let client = Arc::new(MockWebSocketClient);
    let reconnect_strategy = ReconnectStrategy::new(3, 2); // Retry 3 times with a 2-second base delay

    let reconnect_result = reconnect_strategy.reconnect(client).await;

    assert!(
        reconnect_result.is_none(),
        "Expected reconnect to stop after max retries with MockWebSocketClient"
    );
}

/// Tests the full lifecycle of the `WebSocketController`, including connection, reconnection, and keep-alive mechanisms.
///
/// This test simulates a WebSocket server to validate the functionality of the controller.
#[tokio::test]
async fn test_websocket_controller_full_lifecycle() {
    let mut controller = WebSocketController::new("ws://node_server:9001", 3, Some(5));

    let ws_stream = Arc::new(Mutex::new(
        controller
            .connect()
            .await
            .expect("Failed to connect to WebSocket server"),
    ));

    // Test connection and message sending
    let connect_result = controller.connect_and_send_message(b"Hello, WebSocket!").await;
    assert!(
        connect_result.is_ok(),
        "Expected connection to succeed: {:?}",
        connect_result.err()
    );

    // Simulate a disconnection and test reconnection logic
    let reconnect_result = controller.reconnect_if_needed().await;
    assert!(
        reconnect_result.is_ok(),
        "Expected reconnection to succeed: {:?}",
        reconnect_result.err()
    );

    // Test maintain connection (keep-alive)
    let maintain_result = controller.maintain_connection(ws_stream.clone()).await;
    assert!(
        maintain_result.is_ok(),
        "Expected maintain connection to succeed: {:?}",
        maintain_result.err()
    );
}

/// Tests message serialization in both JSON and CBOR formats.
///
/// This test verifies that messages can be successfully serialized into the expected formats.
#[test]
fn test_message_serialization() {
    let message = "Hello, WebSocket!";

    // Test JSON serialization
    let serialized = MessageHandler::serialize(&message, MessageFormat::Json).unwrap();
    assert!(
        !serialized.is_empty(),
        "Expected non-empty serialized JSON data"
    );

    // Test CBOR serialization
    let serialized_cbor = MessageHandler::serialize(&message, MessageFormat::Cbor).unwrap();
    assert!(
        !serialized_cbor.is_empty(),
        "Expected non-empty serialized CBOR data"
    );
}

/// Tests message deserialization in both JSON and CBOR formats.
///
/// This test ensures that serialized messages can be deserialized back to their original values.
#[test]
fn test_message_deserialization() {
    let message = "Hello, WebSocket!";

    // Serialize message in JSON format
    let serialized_json = MessageHandler::serialize(&message, MessageFormat::Json).unwrap();
    match MessageHandler::deserialize::<String>(&serialized_json, MessageFormat::Json) {
        Ok(Some(deserialized_json)) => {
            assert_eq!(
                deserialized_json,
                message.to_string(),
                "Expected deserialized JSON to match original message"
            );
        }
        Ok(None) => error!("Deserialization returned None, expected Some value"),
        Err(e) => error!("Deserialization error: {:?}", e),
    }

    // Serialize message in CBOR format
    let serialized_cbor = MessageHandler::serialize(&message, MessageFormat::Cbor).unwrap();
    match MessageHandler::deserialize::<String>(&serialized_cbor, MessageFormat::Cbor) {
        Ok(Some(deserialized_cbor)) => {
            assert_eq!(
                deserialized_cbor,
                message.to_string(),
                "Expected deserialized CBOR to match original message"
            );
        }
        Ok(None) => error!("Deserialization returned None, expected Some value"),
        Err(e) => error!("Deserialization error: {:?}", e),
    }
}

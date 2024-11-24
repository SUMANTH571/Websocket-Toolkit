#![no_main]

//! Fuzz testing for the `websocket_toolkit` crate.
//!
//! This fuzz target performs various tests to ensure the robustness of the WebSocket implementation
//! and its associated functionalities. It tests:
//! 1. Message deserialization with random inputs.
//! 2. WebSocket connection handling under various conditions.
//! 3. Reconnection logic with retry strategies.
//! 4. Keep-alive mechanisms with randomized inputs.

use libfuzzer_sys::fuzz_target;
use websocket_toolkit::messages::{MessageHandler, MessageFormat};
use websocket_toolkit::connection::WebSocketClient;
use websocket_toolkit::reconnection::ReconnectStrategy;
use websocket_toolkit::keep_alive::KeepAlive;
use arbitrary::{Arbitrary, Unstructured};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::time::{timeout, Duration};
use tungstenite::error::Error as TungsteniteError;
use log::{debug, error};

/// Mock WebSocket client used for fuzz testing.
///
/// This client simulates connection failures to test reconnection and error-handling logic.
struct MockWebSocketClient;

impl MockWebSocketClient {
    /// Simulates a connection attempt.
    ///
    /// # Returns
    /// - `Err(TungsteniteError::ConnectionClosed)`: Simulates a connection failure.
    pub fn connect(&self) -> Result<(), TungsteniteError> {
        Err(TungsteniteError::ConnectionClosed)
    }
}

/// Fuzzing target structure for generating random test data.
///
/// This structure contains random message content and format for testing WebSocket functionalities.
///
/// # Fields
/// - `content`: Random binary data for messages.
/// - `format`: Message format (JSON or CBOR).
#[derive(Arbitrary, Debug)]
struct FuzzData {
    /// Random binary data to simulate message content.
    content: Vec<u8>,
    /// Randomly chosen message format (JSON or CBOR).
    format: MessageFormat,
}

/// Fuzzing target for testing WebSocket functionalities.
///
/// This fuzz target performs the following operations:
/// 1. Deserializes messages using random data.
/// 2. Tests WebSocket connection establishment.
/// 3. Tests reconnection logic with retries.
/// 4. Tests the keep-alive mechanism under fuzzed conditions.
///
/// # Arguments
/// - `data`: Random byte slice provided by the fuzzer.
fuzz_target!(|data: &[u8]| {
    let mut unstructured = Unstructured::new(data);

    // Attempt to generate fuzz data
    if let Ok(fuzz_data) = FuzzData::arbitrary(&mut unstructured) {
        // Validate input size to prevent overloading
        if fuzz_data.content.is_empty() || fuzz_data.content.len() > 1024 {
            debug!("Skipping invalid input: content too small or too large");
            return;
        }

        // Create a Tokio runtime for asynchronous testing
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");

        runtime.block_on(async {
            // Initialize WebSocket client
            let client = WebSocketClient::new("ws://127.0.0.1:9001", 3);

            // 1. **Fuzz Message Deserialization**
            // Test deserialization of random data into a `String` using the specified format
            let _ = MessageHandler::deserialize::<String>(&fuzz_data.content, fuzz_data.format);

            // 2. **Fuzz WebSocket Connection**
            let mut ws_stream = match timeout(Duration::from_secs(1), client.connect()).await {
                Ok(Ok(stream)) => stream,
                Ok(Err(e)) => {
                    debug!("WebSocket connection failed: {:?}", e);
                    return;
                }
                Err(_) => {
                    debug!("WebSocket connection timed out.");
                    return;
                }
            };

            // Send a fuzzed message using the WebSocket client
            let _ = client
                .send_message(&mut ws_stream, &String::from_utf8_lossy(&fuzz_data.content))
                .await;

            // 3. **Fuzz Reconnection Logic**
            let reconnect_strategy = ReconnectStrategy::new(2, 1); // Retry twice with a 1-second base delay
            let mock_client = Arc::new(MockWebSocketClient);
            for _ in 0..reconnect_strategy.get_retries() {
                let _ = mock_client.connect();
            }

            // 4. **Fuzz Keep-Alive Mechanism**
            let keep_alive = KeepAlive::new(Duration::from_millis(500)); // 500 ms ping interval
            let _ = timeout(Duration::from_secs(1), keep_alive.start(&mut ws_stream)).await;
        });
    }
});

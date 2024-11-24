#![allow(unused_imports)]
#![allow(unused_variables)]

//! A simple WebSocket client example using the `websocket_toolkit` crate.
//!
//! This example demonstrates how to:
//! 1. Establish a WebSocket connection using `WebSocketController`.
//! 2. Send and receive messages in JSON and CBOR formats.
//! 3. Maintain the connection using a keep-alive mechanism.
//! 4. Simulate reconnection logic after a server disconnect.

use websocket_toolkit::controller::WebSocketController;
use tokio::time::{timeout, Duration, sleep};
use log::{info, error};
use env_logger;
use serde::{Deserialize, Serialize};
use serde_cbor;
use tokio::sync::Mutex;
use std::sync::Arc;

/// A struct representing a WebSocket message with a type and content.
///
/// # Fields
/// - `msg_type`: The type of the message, e.g., "greeting".
/// - `content`: The actual content of the message.
#[derive(Serialize, Deserialize, Debug)]
struct Message {
    /// The type of the message (e.g., "request", "response").
    #[serde(rename = "type")]
    msg_type: String,
    /// The content of the message.
    content: String,
}

/// Entry point for the example demonstrating WebSocket functionality.
///
/// # Steps
/// 1. Initializes the logger.
/// 2. Establishes a connection to the WebSocket server.
/// 3. Sends test messages in JSON and CBOR formats.
/// 4. Simulates keep-alive and reconnection logic.
///
/// # Returns
/// A `Result` indicating success or failure.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let url = "ws://node_server:9001";
    let retries = 3;
    let ping_interval = Some(5); // Ping every 5 seconds for keep-alive
    let mut controller = WebSocketController::new(url, retries, ping_interval);

    info!("Attempting to connect...");
    let ws_stream = match timeout(Duration::from_secs(5), controller.connect()).await {
        Ok(Ok(stream)) => {
            info!("Connection successful.");
            Arc::new(Mutex::new(stream))
        }
        Ok(Err(e)) => {
            error!("Connection failed: {}", e);
            return Ok(());
        }
        Err(_) => {
            error!("Connection timed out.");
            return Ok(());
        }
    };

    // Test sending JSON and CBOR messages
    send_test_messages(&mut controller, &ws_stream).await?;

    // Simulate keep-alive and reconnection logic
    simulate_keep_alive_and_reconnect(&mut controller, ws_stream, ping_interval).await?;

    info!("Example successfully tested. Closing connection and exiting.");
    Ok(())
}

/// Sends test messages in JSON and CBOR formats.
///
/// # Arguments
/// - `controller`: The `WebSocketController` managing the WebSocket connection.
/// - `ws_stream`: A thread-safe shared WebSocket stream.
///
/// # Returns
/// A `Result` indicating success or failure.
async fn send_test_messages(
    controller: &mut WebSocketController,
    ws_stream: &Arc<Mutex<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // JSON Message
    let json_message = serde_json::to_vec(&Message {
        msg_type: "greeting".to_string(),
        content: "Hello, server (JSON)!".to_string(),
    })?;
    controller
        .send_message(&mut *ws_stream.lock().await, json_message.as_slice())
        .await?;
    info!("Sent JSON message.");

    // CBOR Message
    let cbor_message = serde_cbor::to_vec(&Message {
        msg_type: "greeting".to_string(),
        content: "Hello, server (CBOR)!".to_string(),
    })?;
    controller
        .send_message(&mut *ws_stream.lock().await, cbor_message.as_slice())
        .await?;
    info!("Sent CBOR message.");

    Ok(())
}

/// Simulates keep-alive functionality and reconnection logic.
///
/// # Arguments
/// - `controller`: The `WebSocketController` managing the WebSocket connection.
/// - `ws_stream`: A thread-safe shared WebSocket stream.
/// - `ping_interval`: Optional interval in seconds for sending keep-alive pings.
///
/// # Returns
/// A `Result` indicating success or failure.
async fn simulate_keep_alive_and_reconnect(
    controller: &mut WebSocketController,
    ws_stream: Arc<Mutex<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>,
    ping_interval: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Start keep-alive if the interval is set
    if let Some(interval) = ping_interval {
        controller.maintain_connection(ws_stream.clone()).await?;
    }

    for _ in 0..3 {
        let mut stream = ws_stream.lock().await;
        match controller.receive_message(&mut *stream).await {
            Ok(Some(msg)) => {
                if let Ok(json_msg) = serde_json::from_slice::<Message>(&msg) {
                    info!("Received JSON: {:?}", json_msg);
                } else if let Ok(cbor_msg) = serde_cbor::from_slice::<Message>(&msg) {
                    info!("Received CBOR: {:?}", cbor_msg);
                } else {
                    error!(
                        "Unsupported message format: {:?}",
                        String::from_utf8_lossy(&msg)
                    );
                }
            }
            Ok(None) => continue, // Ignore control messages like Ping/Pong
            Err(e) => {
                error!("Error receiving message: {}", e);
                break;
            }
        }

        sleep(Duration::from_secs(ping_interval.unwrap_or(5))).await;
    }

    info!("Simulating server disconnect...");
    controller.disconnect().await?;
    sleep(Duration::from_secs(2)).await;

    info!("Reconnecting...");
    match controller.connect().await {
        Ok(new_stream) => {
            let ws_stream = Arc::new(Mutex::new(new_stream));
            info!("Reconnected successfully!");
        }
        Err(e) => error!("Reconnection failed: {}", e),
    }

    Ok(())
}

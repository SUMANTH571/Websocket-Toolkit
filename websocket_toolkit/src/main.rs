#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use websocket_toolkit::controller::WebSocketController;
use tokio::time::{timeout, Duration, sleep};
use log::{info, error};
use env_logger;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_cbor;
use tokio::sync::Mutex;
use std::sync::Arc;

/// A message structure used for communication over WebSocket.
///
/// This struct is used for serializing and deserializing both JSON and CBOR
/// messages. It represents a message containing a type and a content field.
#[derive(Serialize, Deserialize, Debug)]
struct Message {
    /// The type of the message, such as "request" or "response".
    #[serde(rename = "type")]
    msg_type: String,
    /// The actual content of the message.
    content: String,
}

/// Main entry point for the WebSocket client application.
///
/// This function initializes the logger, configures the WebSocket controller,
/// and manages exponential backoff for reconnection attempts. It serves as the
/// primary loop for establishing and maintaining a WebSocket connection.
#[tokio::main]
async fn main() {
    // Initialize the logging framework for structured logs.
    env_logger::init();

    // Configuration variables for the WebSocket client.
    let url = "ws://127.0.0.1:9001";
    let retries = 5; // Maximum number of reconnection attempts.
    let ping_interval = Some(5); // Interval in seconds for keep-alive pings.

    // Instantiate a WebSocket controller to manage the connection.
    let mut controller = WebSocketController::new(url, retries, ping_interval);

    // Exponential backoff for reconnection attempts.
    let mut backoff = 1;
    loop {
        info!("Attempting to connect...");
        match timeout(Duration::from_secs(5), controller.connect()).await {
            Ok(Ok(ws_stream)) => {
                info!("Connected to WebSocket server!");
                let ws_stream = Arc::new(Mutex::new(ws_stream));

                // Run the connection loop to send/receive messages.
                if let Err(e) = run_connection_loop(&mut controller, ws_stream.clone(), ping_interval).await {
                    error!("Connection loop error: {}", e);
                }
            }
            Ok(Err(e)) => error!("Connection attempt failed: {}", e),
            Err(_) => error!("Connection timed out."),
        }

        if backoff > 30 {
            error!("Maximum retries reached. Exiting...");
            break;
        }

        error!("Reconnecting in {} seconds...", backoff);
        sleep(Duration::from_secs(backoff)).await;
        backoff = (backoff * 2).min(30); // Exponential backoff logic.
    }
}

/// Handles the main WebSocket connection loop.
///
/// This function is responsible for maintaining the WebSocket connection,
/// including sending and receiving messages, handling keep-alive pings, and
/// processing incoming messages in JSON or CBOR format.
///
/// # Arguments
///
/// * `controller` - The `WebSocketController` instance managing the connection.
/// * `ws_stream` - A thread-safe shared WebSocket stream wrapped in `Arc<Mutex<_>>`.
/// * `ping_interval` - An optional interval (in seconds) for sending keep-alive pings.
///
/// # Returns
///
/// * `Ok(())` - On successful completion of the connection loop.
/// * `Err(Box<dyn std::error::Error>)` - If an error occurs during message processing or pinging.
///
/// # Errors
///
/// Returns an error if any WebSocket operation (such as receiving, sending,
/// or pinging) fails.
async fn run_connection_loop(
    controller: &mut WebSocketController,
    ws_stream: Arc<Mutex<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>,
    ping_interval: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Start the keep-alive mechanism.
    controller.maintain_connection(ws_stream.clone()).await?;

    loop {
        let mut stream = ws_stream.lock().await;

        // Receive a message from the WebSocket server.
        match controller.receive_message(&mut *stream).await {
            Ok(Some(msg)) => {
                // Attempt to deserialize as JSON message.
                if let Ok(json_msg) = serde_json::from_slice::<Message>(&msg) {
                    info!("Received JSON message: {:?}", json_msg);
                }
                // Attempt to deserialize as CBOR message.
                else if let Ok(cbor_msg) = serde_cbor::from_slice::<Message>(&msg) {
                    info!("Received CBOR message: {:?}", cbor_msg);
                }
                // Handle unknown or unsupported message formats.
                else {
                    error!("Received unknown message format");
                }

                // Send an acknowledgment response in CBOR format.
                let cbor_response = serde_cbor::to_vec(&Message {
                    msg_type: "response".to_string(),
                    content: "Acknowledged (CBOR)".to_string(),
                })?;
                controller.send_message(&mut *stream, &cbor_response).await?;
            }
            Ok(None) => info!("Control message received, ignoring."),
            Err(e) => {
                error!("Error receiving message: {}", e);
                break;
            }
        }

        // Send keep-alive ping at the specified interval.
        if let Some(interval) = ping_interval {
            if let Err(e) = controller.send_ping(&mut *stream).await {
                error!("Ping failed: {}", e);
                break;
            }
            sleep(Duration::from_secs(interval)).await;
        }
    }
    Ok(())
}

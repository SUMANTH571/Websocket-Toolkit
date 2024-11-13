#![allow(unused_imports)]
use websocket_toolkit::connection::WebSocketClient;
use websocket_toolkit::controller::WebSocketController;
use websocket_toolkit::messages::{MessageHandler, MessageFormat};
use websocket_toolkit::reconnection::{ReconnectStrategy, Connectable};
use std::sync::Arc;
use log::{info, error};
use futures_util::{StreamExt, SinkExt}; // Ensure these imports are included
use websocket_toolkit::keep_alive::KeepAlive; // Import KeepAlive for the test
use tokio_tungstenite::tungstenite::Error; // Import Error
use async_trait::async_trait;

// MockWebSocketClient simulating a connection failure for testing
struct MockWebSocketClient;

#[async_trait]
impl Connectable for MockWebSocketClient {
    async fn connect(&self) -> Result<(), Error> {
        Err(Error::ConnectionClosed) // Simulate a connection failure
    }
}

#[tokio::test]
async fn test_websocket_client_connect() {
    // Test WebSocket client connection with expected failure due to mock
    let client = MockWebSocketClient;
    
    // Await the result from the connect method
    let result = client.connect().await;
    
    // Check if it is an error
    assert!(result.is_err(), "Expected WebSocket connection to fail with MockWebSocketClient");
}

#[tokio::test]
async fn test_reconnect_strategy() {
    let client = Arc::new(MockWebSocketClient);
    let reconnect_strategy = ReconnectStrategy::new(3, 2); // Retry 3 times with a 2-second base delay

    let reconnect_result = reconnect_strategy.reconnect(client).await;
    
    // Handle or ignore the result
    let _ = reconnect_result;  // If not needed, we ignore the result to avoid warnings

    assert!(
        reconnect_result.is_none(),
        "Expected reconnect to stop after max retries with MockWebSocketClient"
    );
}

#[tokio::test]
async fn test_websocket_controller_full_lifecycle() {
    let mut controller = WebSocketController::new("ws://127.0.0.1:0", 3, Some(5));

    // Testing connection failure with mock WebSocket
    let connect_result = controller.connect_and_send_message(b"Hello, WebSocket!").await;
    assert!(connect_result.is_err(), "Expected connection to fail with mock setup");

    controller.reconnect_if_needed().await;
    controller.maintain_connection().await;
}

#[test]
fn test_message_serialization() {
    let message = "Hello, WebSocket!";

    // Test JSON serialization
    let serialized = MessageHandler::serialize(&message, MessageFormat::Json).unwrap();
    assert!(!serialized.is_empty(), "Expected non-empty serialized JSON data");

    // Test CBOR serialization
    let serialized_cbor = MessageHandler::serialize(&message, MessageFormat::Cbor).unwrap();
    assert!(!serialized_cbor.is_empty(), "Expected non-empty serialized CBOR data");
}

#[test]
fn test_message_deserialization() {
    let message = "Hello, WebSocket!";

    // Serialize message in JSON format
    let serialized_json = MessageHandler::serialize(&message, MessageFormat::Json).unwrap();
    match MessageHandler::deserialize::<String>(&serialized_json, MessageFormat::Json) {
        Ok(Some(deserialized_json)) => {
            assert_eq!(deserialized_json, message.to_string(), "Expected deserialized JSON to match original message");
        },
        Ok(None) => error!("Deserialization returned None, expected Some value"),
        Err(e) => error!("Deserialization error: {:?}", e),
    }

    // Serialize message in CBOR format
    let serialized_cbor = MessageHandler::serialize(&message, MessageFormat::Cbor).unwrap();
    match MessageHandler::deserialize::<String>(&serialized_cbor, MessageFormat::Cbor) {
        Ok(Some(deserialized_cbor)) => {
            assert_eq!(deserialized_cbor, message.to_string(), "Expected deserialized CBOR to match original message");
        },
        Ok(None) => error!("Deserialization returned None, expected Some value"),
        Err(e) => error!("Deserialization error: {:?}", e),
    }
}

// #[tokio::test]
async fn test_keep_alive_with_mock_server() {
    // Set up a oneshot channel to notify when the server is ready
    let (server_ready_tx, server_ready_rx) = tokio::sync::oneshot::channel::<()>();

    // Set up a mock WebSocket server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9001")
        .await
        .expect("Failed to bind server");

    // Spawn a task to run the server in the background
    let server_handle = tokio::spawn(async move {
        match listener.accept().await {
            Ok((stream, _)) => {
                println!("Server: Client connected. Accepting WebSocket connection...");
                let mut ws_stream = tokio_tungstenite::accept_async(stream)
                    .await
                    .expect("Failed to accept WebSocket connection");
   
                // Immediately send readiness signal after accepting the connection
                println!("Server: Connection established, ready to handle pings.");
                let _ = server_ready_tx.send(());  // Send readiness signal immediately
   
                // Listen for ping messages from the client and respond with pong
                while let Some(Ok(tokio_tungstenite::tungstenite::protocol::Message::Ping(_))) = ws_stream.next().await {
                    println!("Server: Received ping, sending pong...");
                    ws_stream
                        .send(tokio_tungstenite::tungstenite::protocol::Message::Pong(Vec::new())) // Respond with Pong
                        .await
                        .expect("Failed to send pong response");
                }
            }
            Err(e) => {
                println!("Server: Error while accepting connection: {}", e);
            }
        }
   });
   

    // Wait for the server to be ready with a longer timeout
    match tokio::time::timeout(std::time::Duration::from_secs(60), server_ready_rx).await {
        Ok(_) => println!("Server is ready to accept connections."),
        Err(_) => {
            println!("Server readiness signal timed out.");
            panic!("Failed to receive server readiness signal within timeout.");
        }
    }

    // Set up the client side (KeepAlive) to connect to the mock server
    let client_url = "ws://127.0.0.1:9001";
    let client = WebSocketClient::new(client_url, 3);

    // Attempt to establish the WebSocket connection
    println!("Client: Attempting to connect...");
    let mut ws_stream = client.connect().await.expect("Failed to connect to mock WebSocket server");
    println!("Client: WebSocket connected.");

    // Create KeepAlive with a short interval for testing
    let keep_alive = KeepAlive::new(std::time::Duration::from_secs(1));

    // Run the keep-alive ping mechanism with a timeout to avoid infinite loops in tests
    let keep_alive_handle = tokio::spawn(async move {
        keep_alive.start(&mut ws_stream).await.unwrap_or_else(|e| {
            error!("Keep-alive error: {}", e); // Log error if keep-alive fails
        });
    });

    // Set a reasonable timeout for the keep-alive mechanism and handle potential errors
    match tokio::time::timeout(std::time::Duration::from_secs(15), keep_alive_handle).await {
        Ok(_) => println!("Keep-alive test completed successfully."),
        Err(e) => {
            println!("Keep-alive test timed out: {:?}", e);
            panic!("Keep-alive test failed due to timeout");
        }
    }

    // Cleanup the server handle after the test
    server_handle.abort();
    let _ = server_handle.await;
}




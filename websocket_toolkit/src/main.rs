use websocket_toolkit::controller::WebSocketController;
use log::{info, error};
use env_logger;

#[tokio::main] // This macro automatically sets up the Tokio runtime
async fn main() {
    // Initialize the logger for structured output (recommended for async applications)
    env_logger::init();

    // Define WebSocket server URL and configuration parameters
    let url = "wss://example.com/socket";
    let retries = 3;            // Number of reconnection attempts
    let ping_interval = Some(10); // Interval for keep-alive pings in seconds

    // Create a new WebSocketController with the given URL, retries, and ping interval
    let mut controller = WebSocketController::new(url, retries, ping_interval);

    // Attempt to connect to the WebSocket server and send a message
    info!("Connecting to WebSocket server and sending a message...");
    if let Err(e) = controller.connect_and_send_message(b"Hello, WebSocket!").await {
        error!("Failed to connect and send message: {}", e);
        return; // Exit early if connection and message send fails
    }

    // Handle reconnection logic if the connection is lost
    info!("Checking if reconnection is needed...");
    controller.reconnect_if_needed().await;

    // Start the keep-alive mechanism to maintain an active connection
    info!("Starting keep-alive mechanism...");
    controller.maintain_connection().await;

    info!("WebSocket session completed.");
}

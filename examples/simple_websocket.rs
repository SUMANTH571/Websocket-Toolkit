use websocket_toolkit::controller::WebSocketController;
use tokio::time::{timeout, Duration, sleep};
use log::{info, error};
use env_logger;

#[tokio::main]
async fn main() {
    env_logger::init();  // Initialize logging

    let url = "ws://127.0.0.1:9001";  // WebSocket URL
    let retries = 3;  // Number of reconnection attempts
    let ping_interval = Some(5); // 5-second interval for keep-alive

    let mut controller = WebSocketController::new(url, retries, ping_interval);

    // Attempt connection and send message
    loop {
        info!("Attempting to connect to WebSocket server...");
        match timeout(Duration::from_secs(5), controller.connect_and_send_message(b"Hello, WebSocket!")).await {
            Ok(Ok(_)) => {
                info!("Message sent successfully. Connection established.");
                break; // Exit the loop if connection is successful
            }
            Ok(Err(e)) => {
                error!("Failed to send message: {}", e);
                // Allow retrying in the next loop
            },
            Err(_) => {
                error!("Timeout reached while sending message.");
                // Allow retrying in the next loop
            },
        }
        info!("Retrying connection in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }

    // Keep-alive and reconnection loop
    loop {
        if let Err(e) = timeout(Duration::from_secs(60), controller.maintain_connection()).await {
            error!("Keep-alive mechanism encountered an error: {:?}", e);
            info!("Attempting reconnection...");

            // Attempt to reconnect if keep-alive fails
            if let Err(reconnect_err) = controller.reconnect_if_needed().await {
                error!("Failed to reconnect: {}", reconnect_err);
                info!("Retrying connection in 5 seconds...");
                sleep(Duration::from_secs(5)).await;
                continue; // Continue to attempt reconnection
            }

            info!("Reconnected successfully!");
        }
        break; // Exit the loop if connection is successfully maintained
    }
    info!("WebSocket session completed.");
}

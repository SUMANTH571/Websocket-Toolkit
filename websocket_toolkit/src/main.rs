use websocket_toolkit::controller::WebSocketController;
use tokio::time::{timeout, Duration, sleep};
use log::{info, error};
use env_logger;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_cbor;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    #[serde(rename = "type")]
    msg_type: String,
    content: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let url = "ws://127.0.0.1:9001";
    let retries = 5; // Maximum retries
    let ping_interval = Some(5); // Keep-alive interval (seconds)

    let mut controller = WebSocketController::new(url, retries, ping_interval);

    // Exponential backoff for reconnection
    let mut backoff = 1;
    loop {
        info!("Attempting to connect...");
        match timeout(Duration::from_secs(5), controller.connect()).await {
            Ok(Ok(ws_stream)) => {
                info!("Connected to WebSocket server!");
                let ws_stream = Arc::new(Mutex::new(ws_stream));

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
        backoff = (backoff * 2).min(30); // Exponential backoff
    }
}

async fn run_connection_loop(
    controller: &mut WebSocketController,
    ws_stream: Arc<Mutex<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>,
    ping_interval: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Start keep-alive
    controller.maintain_connection(ws_stream.clone()).await?;

    loop {
        let mut stream = ws_stream.lock().await;
        match controller.receive_message(&mut *stream).await {
            Ok(Some(msg)) => {
                // Handle JSON messages
                if let Ok(json_msg) = serde_json::from_slice::<Message>(&msg) {
                    info!("Received JSON message: {:?}", json_msg);
                }
                // Handle CBOR messages
                else if let Ok(cbor_msg) = serde_cbor::from_slice::<Message>(&msg) {
                    info!("Received CBOR message: {:?}", cbor_msg);
                }
                // Handle unknown message format
                else {
                    error!("Received unknown message format");
                }

                // Send a CBOR response
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

        // Send keep-alive ping
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

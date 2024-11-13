#![allow(unused_imports)]
use crate::connection::WebSocketClient;
use crate::messages::{MessageHandler, MessageFormat};
use crate::reconnection::ReconnectStrategy;
use crate::keep_alive::KeepAlive;
use log::{info, error, debug, warn};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{sink::SinkExt, StreamExt};  // Import StreamExt here
use tokio::time::{interval, sleep, Duration};
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;


pub struct WebSocketController {
    client: Arc<WebSocketClient>,
    reconnect_strategy: ReconnectStrategy,
    keep_alive: Option<KeepAlive>,  // keep_alive field used for keep-alive functionality
    ping_interval: u64,     // Add ping_interval field (in seconds)
    retries: u32,   
}

impl WebSocketController {
    pub fn new(url: &str, retries: u32, ping_interval: Option<u64>) -> Self {
        let client = Arc::new(WebSocketClient::new(url, retries));
        let reconnect_strategy = ReconnectStrategy::new(retries, 2);  // 2 seconds base delay
        let keep_alive = ping_interval.map(|interval| KeepAlive::new(Duration::from_secs(interval)));

        WebSocketController {
            client,
            reconnect_strategy,
            keep_alive,
            ping_interval: ping_interval.unwrap_or(10), // Default to 10 seconds if None
            retries,
        }
    }

    pub async fn connect_and_send_message(&self, message: &[u8]) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        let mut ws_stream = self.client.connect().await?;
        self.send_message(&mut ws_stream, message).await;
        Ok(())
    }
  
    async fn send_message(
        &self,
        ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
        message: &[u8]
    ) {
        let msg = if let Ok(text) = std::str::from_utf8(message) {
            Message::Text(text.to_string())
        } else {
            Message::Binary(message.to_vec())
        };
    
        if let Err(e) = ws_stream.send(msg).await {
            error!("Failed to send message: {}", e);
        }
    }

    pub async fn reconnect_if_needed(&self) -> Result<(), String> {
        // Handle the Result properly
        match self.reconnect_strategy.reconnect(self.client.clone()).await {
            Some(()) => Ok(()),
            None => Err("Failed to reconnect after maximum retries".to_string()),
        }
    }

    pub async fn maintain_connection(&mut self) {
        let mut ws_stream = match self.client.connect().await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Initial connection failed: {}", e);
                return;
            }
        };

        let mut ping_interval = interval(Duration::from_secs(self.ping_interval));
        let mut retry_count = 0;

        loop {
            tokio::select! {
                _ = ping_interval.tick() => {
                    match ws_stream.send(Message::Ping(vec![])).await {
                        Ok(_) => info!("Ping sent to keep connection alive"),
                        Err(e) => {
                            error!("Failed to send ping: {}", e);
                            retry_count += 1;

                            // Backoff strategy with maximum retries
                            if retry_count > self.retries {
                                error!("Maximum reconnection attempts reached. Exiting keep-alive.");
                                break;
                            }

                            // Attempt to reconnect after a delay
                            error!("Attempting reconnection in 5 seconds...");
                            sleep(Duration::from_secs(5)).await;
                            ws_stream = match self.client.connect().await {
                                Ok(new_stream) => {
                                    retry_count = 0; // Reset retries on successful reconnect
                                    new_stream
                                },
                                Err(reconnect_error) => {
                                    error!("Reconnection attempt failed: {}", reconnect_error);
                                    continue;
                                }
                            };
                        }
                    }
                }
            }
        }
    }

    pub async fn setup_mock_server() -> SocketAddr {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind server");
        let addr = listener.local_addr().unwrap();

        debug!("Mock WebSocket server is listening at: {}", addr);

        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await 
            {
                debug!("Received connection from client...");
                let mut ws_stream = accept_async(stream)
                    .await
                    .expect("Failed to accept WebSocket connection");

                while let Some(msg) = ws_stream.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            debug!("Server received message: {}", text);
                            ws_stream.send(Message::Text("Echo".to_string())).await.unwrap();
                        }
                        Ok(Message::Ping(ping)) => {
                            ws_stream.send(Message::Pong(ping)).await.unwrap();
                        }
                        _ => (),
                    }
                }
            }
        });

        addr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    // Example test using reconnect and handling the result
    #[tokio::test]
    async fn test_websocket_controller_lifecycle() {
        let mut controller = WebSocketController::new("ws://127.0.0.1:9001", 3, Some(10));
        
        // Make sure to handle the result properly in the main code
        if let Err(e) = controller.connect_and_send_message(b"Hello, WebSocket!").await {
            error!("Failed to connect and send message: {}", e);
        }
        controller.reconnect_if_needed().await.unwrap_or_else(|e| error!("{}", e));
        controller.maintain_connection().await;
    }
}

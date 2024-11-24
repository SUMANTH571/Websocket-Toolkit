#![allow(unused_imports)]

use crate::connection::WebSocketClient;
use crate::messages::{MessageHandler, MessageFormat};
use crate::reconnection::ReconnectStrategy;
use crate::keep_alive::KeepAlive;
use log::{info, error, debug, warn};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{sink::SinkExt, StreamExt};
use tokio::time::{sleep, Duration};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::error::Error as StdError;

pub struct WebSocketController {
    client: Arc<WebSocketClient>,
    reconnect_strategy: Option<ReconnectStrategy>,
    ping_interval: Duration,
    retries: u32,
}

impl WebSocketController {
    pub fn new(url: &str, retries: u32, ping_interval: Option<u64>) -> Self {
        Self {
            client: Arc::new(WebSocketClient::new(url, retries)),
            reconnect_strategy: Some(ReconnectStrategy::new(retries, 2)),
            ping_interval: Duration::from_secs(ping_interval.unwrap_or(5)),
            retries,
        }
    }

    pub async fn connect(
        &self,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn StdError>> {
        self.client
            .connect()
            .await
            .map_err(|e| Box::new(e) as Box<dyn StdError>)
    }

    pub async fn connect_and_send_message(
        &mut self,
        message: &[u8],
    ) -> Result<(), Box<dyn StdError>> {
        let mut ws_stream = self.connect().await?;
        self.send_message(&mut ws_stream, message).await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), Box<dyn StdError>> {
        self.client.disconnect();
        Ok(())
    }

    pub async fn receive_message(
        &mut self,
        ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> Result<Option<Vec<u8>>, Box<dyn StdError>> {
        if let Some(msg) = ws_stream.next().await {
            match msg? {
                Message::Binary(data) => Ok(Some(data)),
                Message::Text(text) => Ok(Some(text.into_bytes())),
                Message::Ping(_) | Message::Pong(_) => {
                    info!("Received control message: Ping/Pong");
                    Ok(None)
                }
                Message::Close(_) => {
                    info!("Received Close message");
                    Err("Connection closed by server".into())
                }
            }
        } else {
            Err("No message received".into())
        }
    }

    pub async fn send_message(
        &mut self,
        ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
        message: &[u8],
    ) -> Result<(), Box<dyn StdError>> {
        ws_stream.send(Message::Binary(message.to_vec())).await?;
        Ok(())
    }

    pub async fn maintain_connection(
        &self,
        ws_stream: Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    ) -> Result<(), Box<dyn StdError>> {
        let interval = self.ping_interval;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                let mut stream = ws_stream.lock().await;
                if let Err(e) = stream.send(Message::Ping(vec![])).await {
                    error!("Ping failed: {}", e);
                    break;
                }
            }
        });
        Ok(())
    }

    pub async fn reconnect_if_needed(&self) -> Result<(), Box<dyn StdError>> {
        let mut attempts = 0;
        while attempts < self.retries {
            match self.connect().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    error!("Reconnection attempt {} failed: {}", attempts + 1, e);
                    tokio::time::sleep(Duration::from_secs(2_u64.pow(attempts))).await; // Exponential backoff
                    attempts += 1;
                }
            }
        }
        Err("All reconnection attempts failed.".into())
    }
    

    pub async fn send_ping(
        &self,
        ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> Result<(), Box<dyn StdError>> {
        ws_stream.send(Message::Ping(Vec::new())).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;

    // Start a mock WebSocket server for testing
    async fn start_mock_server() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let _ = accept_async(stream).await.unwrap();
            }
        });
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Wait for the server to be ready
        format!("ws://{}", addr)
    }
    

    #[tokio::test]
    async fn test_websocket_controller_lifecycle() -> Result<(), Box<dyn StdError>> {
        let url = "ws://127.0.0.1:9001";
        let mut controller = WebSocketController::new(&url, 3, Some(10));

        // Test connection and sending a message
        let connect_result = controller.connect_and_send_message(b"Hello, WebSocket!").await;
        assert!(
            connect_result.is_ok(),
            "Failed to connect and send message: {:?}",
            connect_result.err()
        );

        // Test reconnection logic
        let reconnect_result = controller.reconnect_if_needed().await;
        assert!(
            reconnect_result.is_ok(),
            "Reconnection failed: {:?}",
            reconnect_result.err()
        );

        // Test maintain connection (keep-alive)
        let ws_stream = Arc::new(Mutex::new(controller.connect().await?));
        controller.maintain_connection(ws_stream.clone()).await?;

        // Simulate activity
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Validate that the connection remains active
        let mut lock = ws_stream.lock().await;
        assert!(
            lock.close(None).await.is_ok(),
            "WebSocket stream failed to close gracefully."
        );

        Ok(())
    }




    #[tokio::test]
    async fn test_websocket_connection() -> Result<(), Box<dyn StdError>> {
        let url = start_mock_server().await;
        let mut controller = WebSocketController::new(&url, 3, Some(5));

        // Test connect method
        let ws_stream = controller.connect().await;
        assert!(
            ws_stream.is_ok(),
            "Connection failed: {:?}",
            ws_stream.err()
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_send_and_receive_message() -> Result<(), Box<dyn StdError>> {
        let url = start_mock_server().await;
        let mut controller = WebSocketController::new(&url, 3, Some(5));
        let mut ws_stream = controller.connect().await.unwrap();

        // Test sending a message
        let message = b"Test Message";
        let send_result = controller.send_message(&mut ws_stream, message).await;
        assert!(
            send_result.is_ok(),
            "Failed to send message: {:?}",
            send_result.err()
        );

        // Mock receiving a message
        let receive_result = controller.receive_message(&mut ws_stream).await;
        assert!(
            receive_result.is_err(),
            "Expected no message, but received one."
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_send_ping() -> Result<(), Box<dyn StdError>> {
        let url = start_mock_server().await;
        let mut controller = WebSocketController::new(&url, 3, Some(5));
        let mut ws_stream = controller.connect().await.unwrap();

        let ping_result = controller.send_ping(&mut ws_stream).await;
        assert!(
            ping_result.is_ok(),
            "Ping failed: {:?}",
            ping_result.err()
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_reconnect_logic() -> Result<(), Box<dyn StdError>> {
        let url = start_mock_server().await;
        let controller = WebSocketController::new(&url, 3, Some(5));

        let reconnect_result = controller.reconnect_if_needed().await;
        assert!(
            reconnect_result.is_ok(),
            "Reconnection failed: {:?}",
            reconnect_result.err()
        );
        Ok(())
    }
}

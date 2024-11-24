use tokio::time::{interval, Duration};
use log::{info, error};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio::net::TcpStream;
use futures_util::sink::SinkExt;

pub struct KeepAlive {
    ping_interval: Duration,
}

impl KeepAlive {
    pub fn new(ping_interval: Duration) -> Self {
        KeepAlive { ping_interval }
    }

    /// Starts sending pings to keep the WebSocket connection alive.
    ///
    /// # Arguments
    ///
    /// * `ws_stream` - The WebSocket stream to send ping messages.
    ///
    /// # Returns
    ///
    /// `Result<(), String>` - Returns an error message on failure.
    pub async fn start(&self, ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> Result<(), String> {
        let mut interval = interval(self.ping_interval);

        loop {
            interval.tick().await;

            match ws_stream.send(Message::Ping(vec![])).await {
                Ok(_) => info!("Ping sent to keep connection alive"),
                Err(e) => {
                    error!("Failed to send ping: {}", e);
                    return Err(format!("Failed to send ping: {}", e)); // Return detailed error message
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio_tungstenite::{accept_async, tungstenite::Message};
    use tokio::time::{timeout, Duration};
    use futures_util::StreamExt;
    use crate::connection::WebSocketClient;
    use tokio::sync::oneshot;
    use tokio::select;

    #[tokio::test]
    async fn test_keep_alive_creation() {
        let keep_alive = KeepAlive::new(Duration::from_secs(10));
        assert_eq!(keep_alive.ping_interval, Duration::from_secs(10));
    }
}

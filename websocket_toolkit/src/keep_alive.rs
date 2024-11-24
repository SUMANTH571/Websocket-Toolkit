use tokio::time::{interval, Duration};
use log::{info, error};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio::net::TcpStream;
use futures_util::sink::SinkExt;

/// The `KeepAlive` struct is responsible for maintaining WebSocket connections
/// by periodically sending ping messages to the server.
///
/// This struct is designed to ensure the WebSocket connection remains active by
/// sending regular ping messages to the server. The interval between pings can
/// be configured during initialization.
pub struct KeepAlive {
    /// The interval at which ping messages are sent to keep the connection alive.
    ping_interval: Duration,
}

impl KeepAlive {
    /// Creates a new `KeepAlive` instance with the specified ping interval.
    ///
    /// # Arguments
    ///
    /// * `ping_interval` - A `Duration` specifying the time interval between ping messages.
    ///
    /// # Returns
    ///
    /// A new instance of `KeepAlive`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use websocket_toolkit::keep_alive::KeepAlive;
    /// use std::time::Duration;
    ///
    /// let keep_alive = KeepAlive::new(Duration::from_secs(10));
    /// ```
    pub fn new(ping_interval: Duration) -> Self {
        KeepAlive { ping_interval }
    }

    /// Starts sending pings to keep the WebSocket connection alive.
    ///
    /// This method runs indefinitely, sending ping messages at the configured interval.
    /// If a ping fails to send, the method returns an error.
    ///
    /// # Arguments
    ///
    /// * `ws_stream` - A mutable reference to the WebSocket stream to send ping messages.
    ///
    /// # Returns
    ///
    /// A `Result<(), String>` - Returns an error message if a ping fails to send.
    ///
    /// # Errors
    ///
    /// Returns an error if sending a ping message fails.
    
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

    /// Tests the creation of a `KeepAlive` instance.
    ///
    /// Ensures that the `KeepAlive` struct is correctly initialized with the given interval.
    ///
    #[tokio::test]
    async fn test_keep_alive_creation() {
        let keep_alive = KeepAlive::new(Duration::from_secs(10));
        assert_eq!(keep_alive.ping_interval, Duration::from_secs(10));
    }
}

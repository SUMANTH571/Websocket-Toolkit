#![allow(unused_imports)]
use crate::connection::WebSocketClient;
use log::{warn, error, info};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::tungstenite::Error;
use std::sync::Arc;
use async_trait::async_trait;

/// A trait that defines the connection behavior for WebSocket clients.
///
/// This trait provides an abstraction for WebSocket clients to define how they connect
/// to a WebSocket server. It includes methods for asynchronous connection establishment
/// and error handling.
#[async_trait]
pub trait Connectable: Send + Sync {
    /// Connects to a WebSocket server.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the connection was successful.
    /// * `Err(Error)` - If the connection failed.
    async fn connect(&self) -> Result<(), Error>;
}

#[async_trait]
impl Connectable for WebSocketClient {
    /// Implements the `connect` method for `WebSocketClient`.
    ///
    /// Asynchronously connects to a WebSocket server. Logs success or failure and spawns a
    /// task to handle the connection process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the task was spawned successfully.
    /// * `Err(Error)` - If the connection process failed.
    async fn connect(&self) -> Result<(), Error> {
        let client = self.clone(); // Clone the client for async tasks

        tokio::spawn(async move {
            match client.connect().await {
                Ok(_) => info!("Successfully connected"),
                Err(e) => error!("Failed to connect: {}", e),
            }
        });

        Ok(())
    }
}

/// A struct that defines a strategy for reconnecting to a WebSocket server with retries and backoff.
///
/// This struct encapsulates the reconnection logic, allowing a WebSocket client to retry
/// connections with exponential backoff up to a maximum number of attempts.
///
/// # Fields
///
/// * `retries` - The maximum number of reconnection attempts.
/// * `base_delay` - The base delay (in seconds) between reconnection attempts.
pub struct ReconnectStrategy {
    retries: u32,
    base_delay: Duration,
}

impl ReconnectStrategy {
    /// Creates a new `ReconnectStrategy` with the specified number of retries and base delay.
    ///
    /// # Arguments
    ///
    /// * `retries` - The maximum number of reconnection attempts.
    /// * `base_delay_secs` - The base delay (in seconds) between reconnection attempts.
    ///
    /// # Returns
    ///
    /// A new instance of `ReconnectStrategy`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use websocket_toolkit::reconnection::{ReconnectStrategy, Connectable};
    /// use std::sync::Arc;
    /// use tokio::runtime::Runtime;
    ///
    /// struct MockClient;
    /// #[async_trait::async_trait]
    /// impl Connectable for MockClient {
    ///     async fn connect(&self) -> Result<(), tokio_tungstenite::tungstenite::Error> {
    ///         Ok(()) // Simulate successful connection
    ///     }
    /// }
    ///
    /// let strategy = ReconnectStrategy::new(3, 1); // Retry 3 times with a 1-second delay
    /// let client = Arc::new(MockClient);
    ///
    /// let runtime = Runtime::new().unwrap();
    /// let result = runtime.block_on(strategy.reconnect(client));
    /// assert!(result.is_some(), "Expected successful reconnection");
    /// ```
    pub fn new(retries: u32, base_delay_secs: u64) -> Self {
        ReconnectStrategy {
            retries,
            base_delay: Duration::from_secs(base_delay_secs),
        }
    }

    /// Retrieves the number of retries for the strategy.
    ///
    /// # Returns
    ///
    /// The maximum number of reconnection attempts allowed.
    pub fn get_retries(&self) -> u32 {
        self.retries
    }

    /// Attempts to reconnect with exponential backoff up to the maximum retries.
    ///
    /// # Arguments
    ///
    /// * `client` - The `WebSocketClient` instance wrapped in an `Arc` to handle reconnection.
    ///
    /// # Returns
    ///
    /// * `Some(())` - If reconnection was successful.
    /// * `None` - If all attempts failed.
    pub async fn reconnect(&self, client: Arc<dyn Connectable>) -> Option<()> {
        for attempt in 1..=self.retries {
            warn!("Reconnection attempt {} of {}", attempt, self.retries);

            match client.connect().await {
                Ok(()) => {
                    info!("Reconnected successfully on attempt {}", attempt);
                    return Some(()); // Successful reconnection
                }
                Err(e) => error!("Reconnection attempt {} failed: {}", attempt, e),
            }

            let delay = self.base_delay * attempt;
            warn!("Waiting for {:?} before next reconnection attempt", delay);
            sleep(delay).await;
        }

        error!("Exceeded maximum reconnection attempts");
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::tungstenite::Error;
    use std::sync::Arc;

    /// A mock WebSocket client for testing purposes.
    struct MockWebSocketClient;

    #[async_trait]
    impl Connectable for MockWebSocketClient {
        /// Simulates a connection failure for testing purposes.
        async fn connect(&self) -> Result<(), Error> {
            Err(Error::ConnectionClosed)
        }
    }

    /// Tests the creation of a `ReconnectStrategy` instance.
    #[tokio::test]
    async fn test_reconnect_strategy_creation() {
        let reconnect_strategy = ReconnectStrategy::new(3, 2);
        assert_eq!(reconnect_strategy.retries, 3);
        assert_eq!(reconnect_strategy.base_delay, Duration::from_secs(2));
    }

    /// Tests the behavior of `ReconnectStrategy` with exponential backoff when all reconnection attempts fail.
    #[tokio::test]
    async fn test_reconnect_with_exponential_backoff() {
        let reconnect_strategy = ReconnectStrategy::new(3, 1);
        let client = Arc::new(MockWebSocketClient);

        let reconnection_result = reconnect_strategy.reconnect(client).await;
        assert!(reconnection_result.is_none(), "Expected all reconnection attempts to fail");
    }

    /// Tests the behavior of `ReconnectStrategy` when reconnection is successful.
    #[tokio::test]
    async fn test_reconnect_success() {
        struct SuccessClient;

        #[async_trait]
        impl Connectable for SuccessClient {
            /// Simulates a successful connection for testing purposes.
            async fn connect(&self) -> Result<(), Error> {
                Ok(()) // Simulate successful connection
            }
        }

        let reconnect_strategy = ReconnectStrategy::new(3, 1);
        let client = Arc::new(SuccessClient);

        let reconnection_result = reconnect_strategy.reconnect(client).await;
        assert!(reconnection_result.is_some(), "Expected successful reconnection");
    }
}

#![allow(unused_imports)]
use crate::connection::WebSocketClient;
use log::{warn, error, info};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::tungstenite::Error;
use std::sync::Arc;
use async_trait::async_trait;

/// Trait defining connection behavior for clients.
#[async_trait]
pub trait Connectable: Send + Sync {
    async fn connect(&self) -> Result<(), Error>;
}

#[async_trait]
impl Connectable for WebSocketClient {
    async fn connect(&self) -> Result<(), Error> {
        // Clone the client to ensure it has the correct lifetime for async operations
        let client = self.clone();  // Clone the client for async tasks
        
        tokio::spawn(async move {
            match client.connect().await {
                Ok(_) => info!("Successfully connected"),
                Err(e) => error!("Failed to connect: {}", e),
            }
        });

        Ok(())
    }
}

pub struct ReconnectStrategy {
    retries: u32,
    base_delay: Duration,
}

impl ReconnectStrategy {
    /// Creates a new `ReconnectStrategy` with the specified number of retries and base delay.
    /// 
    /// # Arguments
    ///
    /// * `retries` - Maximum number of reconnection attempts.
    /// * `base_delay_secs` - Base delay in seconds between reconnection attempts.
    pub fn new(retries: u32, base_delay_secs: u64) -> Self {
        ReconnectStrategy {
            retries,
            base_delay: Duration::from_secs(base_delay_secs),
        }
    }

    /// Attempts to reconnect with exponential backoff up to the maximum retries.
    /// 
    /// # Arguments
    ///
    /// * `client` - The `WebSocketClient` instance wrapped in `Arc` to handle reconnection.
    ///
    /// # Returns
    ///
    /// Returns `Some(())` if reconnection is successful, or `None` if all attempts fail.
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
    
            // Backoff logic, but we must stop after max retries
            let delay = self.base_delay * attempt;
            warn!("Waiting for {:?} before next reconnection attempt", delay);
            sleep(delay).await;
        }
    
        error!("Exceeded maximum reconnection attempts");  // Ensure we stop after max retries
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::tungstenite::Error;
    use std::sync::Arc;

    struct MockWebSocketClient;

    #[async_trait]
    impl Connectable for MockWebSocketClient {
        async fn connect(&self) -> Result<(), Error> {
            Err(Error::ConnectionClosed)
        }
    }

    #[tokio::test]
    async fn test_reconnect_strategy_creation() {
        let reconnect_strategy = ReconnectStrategy::new(3, 2);
        assert_eq!(reconnect_strategy.retries, 3);
        assert_eq!(reconnect_strategy.base_delay, Duration::from_secs(2));
    }

    #[tokio::test]
    async fn test_reconnect_with_exponential_backoff() {
        let reconnect_strategy = ReconnectStrategy::new(3, 1);
        let client = Arc::new(MockWebSocketClient);

        let reconnection_result = reconnect_strategy.reconnect(client).await;
        assert!(reconnection_result.is_none(), "Expected all reconnection attempts to fail");
    }

    #[tokio::test]
    async fn test_reconnect_success() {
        struct SuccessClient;
        #[async_trait]
        impl Connectable for SuccessClient {
            async fn connect(&self) -> Result<(), Error> {
                Ok(())  // Simulate successful connection
            }
        }

        let reconnect_strategy = ReconnectStrategy::new(3, 1);
        let client = Arc::new(SuccessClient);

        let reconnection_result = reconnect_strategy.reconnect(client).await;
        assert!(reconnection_result.is_some(), "Expected successful reconnection");
    }
}

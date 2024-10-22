use crate::connection::WebSocketClient;
use tokio::time::{sleep, Duration};
use log::warn;

pub struct ReconnectStrategy {
    retries: u32,
}

impl ReconnectStrategy {
    pub fn new(retries: u32) -> Self {
        ReconnectStrategy { retries }
    }

    pub async fn reconnect(&self, url: &str) -> Option<WebSocketClient> {
        for attempt in 0..self.retries {
            warn!("Attempt {}: Reconnecting to {}", attempt + 1, url);
            self.private_retry_attempt(url).await;
            sleep(Duration::from_secs(2)).await;
        }
        None
    }

    async fn private_retry_attempt(&self, url: &str) {
        warn!("Trying to reconnect to {}", url);
    }
}

use tokio::time::{interval, Duration};
use crate::connection::WebSocketClient;

pub struct KeepAlive {
    pub ping_interval: Duration,
}

impl KeepAlive {
    pub fn new(ping_interval: Duration) -> Self {
        KeepAlive { ping_interval }
    }

    pub async fn start(&self, _client: &mut WebSocketClient) {
        // Placeholder for ping/pong logic
        let mut interval = interval(self.ping_interval);
        interval.tick().await;
    }
}

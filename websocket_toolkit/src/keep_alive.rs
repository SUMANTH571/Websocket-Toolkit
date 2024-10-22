use tokio::time::{interval, Duration};
use crate::connection::WebSocketClient;
use log::info;

pub struct KeepAlive {
    pub ping_interval: Duration,
}

impl KeepAlive {
    pub fn new(ping_interval: Duration) -> Self {
        KeepAlive { ping_interval }
    }

    pub async fn start(&self, client: &mut WebSocketClient) {
        let mut interval = interval(self.ping_interval);
        loop {
            interval.tick().await;
            self.private_send_ping(client);
        }
    }

    fn private_send_ping(&self, client: &mut WebSocketClient) {
        info!("Sending ping to {}", client.url);
    }
}

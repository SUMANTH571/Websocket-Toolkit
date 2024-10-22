use crate::connection::WebSocketClient;
use crate::messages::{MessageHandler, MessageFormat};
use crate::reconnection::ReconnectStrategy;
use crate::keep_alive::KeepAlive;
use tokio::time::Duration;
use log::info;

pub struct WebSocketController {
    client: WebSocketClient,
    reconnect_strategy: ReconnectStrategy,
    keep_alive: Option<KeepAlive>,
}

impl WebSocketController {
    pub fn new(url: &str, retries: u32, ping_interval: Option<u64>) -> Self {
        let client = WebSocketClient::new(url, retries);
        let reconnect_strategy = ReconnectStrategy::new(retries);
        let keep_alive = ping_interval.map(|interval| KeepAlive::new(Duration::from_secs(interval)));

        WebSocketController {
            client,
            reconnect_strategy,
            keep_alive,
        }
    }

    pub fn connect_and_send_message(&self, message: &str) {
        self.client.connect();
        let serialized = MessageHandler::serialize(&message, MessageFormat::Json);
        info!("Serialized message: {:?}", serialized);
    }

    pub async fn reconnect_if_needed(&self) {
        let _ = self.reconnect_strategy.reconnect(&self.client.url).await;
    }

    pub async fn maintain_connection(&mut self) {
        if let Some(keep_alive) = &mut self.keep_alive {
            keep_alive.start(&mut self.client).await;
        }
    }
}

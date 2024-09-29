use crate::connection::WebSocketClient;

pub struct ReconnectStrategy {
    retries: u32,
}

impl ReconnectStrategy {
    pub fn new(retries: u32) -> Self {
        println!("ReconnectStrategy created with {} retries.", retries);
        Self { retries }
    }

    pub async fn reconnect(&self, url: &str) -> Option<WebSocketClient> {
        for _ in 0..self.retries {
            println!("Attempting to reconnect to {}", url);
            
        }
        None
    }
}

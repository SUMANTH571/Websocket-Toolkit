use log::info;

pub struct WebSocketClient {
    pub url: String,
    retries: u32,
}

impl WebSocketClient {
    pub fn new(url: &str, retries: u32) -> Self {
        WebSocketClient {
            url: url.to_string(),
            retries,
        }
    }

    pub fn connect(&self) {
        self.private_connection_setup();
    }

    pub fn disconnect(&self) {
        self.private_disconnect();
    }

    fn private_connection_setup(&self) {
        info!("Connecting to WebSocket server at {} with {} retries allowed", self.url, self.retries);
    }

    fn private_disconnect(&self) {
        info!("Disconnecting from WebSocket server at {}", self.url);
    }
}

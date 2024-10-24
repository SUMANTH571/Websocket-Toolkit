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

    // Public getter for retries
    pub fn get_retries(&self) -> u32 {
        self.retries
    }

    fn private_connection_setup(&self) {
        println!("Connecting to WebSocket server at {} with {} retries allowed", self.url, self.retries);
    }

    fn private_disconnect(&self) {
        println!("Disconnecting from WebSocket server at {}", self.url);
    }
}


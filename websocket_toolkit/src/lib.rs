pub mod connection;
pub mod reconnection;
pub mod messages;
pub mod keep_alive;

#[cfg(test)]
mod tests {
    use crate::connection::WebSocketClient;


    #[test]
    fn test_websocket_client_creation() {
        let client = WebSocketClient::new("wss://example.com/socket");
        assert_eq!(client.url, "wss://example.com/socket");
    }
}

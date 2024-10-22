use websocket_toolkit::connection::WebSocketClient;

fn main() {
    // Create a new WebSocketClient with URL and retries
    let client = WebSocketClient::new("wss://example.com/socket", 3);
    client.connect();
}

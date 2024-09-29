use websocket_toolkit::connection::WebSocketClient;

fn main() {
    let client = WebSocketClient::new("wss://example.com/socket");
    client.connect();
}

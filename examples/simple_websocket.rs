use websocket_toolkit::controller::WebSocketController;

#[tokio::main]
async fn main() {
    let mut controller = WebSocketController::new("wss://example.com/socket", 3, Some(10));
    controller.connect_and_send_message("Hello, WebSocket!");
    controller.reconnect_if_needed().await;
    controller.maintain_connection().await;
}

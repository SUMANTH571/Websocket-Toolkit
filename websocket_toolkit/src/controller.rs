use crate::connection::WebSocketClient;
use crate::messages::{MessageHandler, MessageFormat};

pub struct WebSocketController {
    client: WebSocketClient,
    message_handler: MessageHandler,
}

impl WebSocketController {
    // Constructor to create a new WebSocketController instance
    pub fn new(url: &str, retries: u32) -> Self {
        let client = WebSocketClient::new(url, retries);
        let message_handler = MessageHandler {};
        WebSocketController { client, message_handler }
    }

    // Public method to connect and send a message using the WebSocket connection
    pub fn connect_and_send_message(&self, message: &str) {
        // Connection to the WebSocket server
        self.client.connect();
        
        // Serialize the message before sending
        let serialized_message = self.message_handler.serialize(&message, MessageFormat::Json);
        
        // Placeholder code for serialized message over the WebSocket
        println!("Serialized message: {:?}", serialized_message);
    }
}

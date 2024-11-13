const WebSocket = require('ws');
const wss = new WebSocket.Server({ port: 9001 });

wss.on('connection', (ws) => {
    // Log when the client connects or reconnects
    console.log('Client connected');
    
    // Handle ping from client and respond with pong
    ws.on('ping', () => {
        console.log('Ping received from client, sending pong');
        ws.pong();  // Send pong back to the client to keep the connection alive
    });

    ws.on('message', (message) => {
        console.log(`Received message: ${message}`);
        ws.send(`Echo: ${message}`); // Echo the message back to the client
    });

    // Log client disconnection
    ws.on('close', () => {
        console.log('Client disconnected');
    });

    // Handle any errors on the connection
    ws.on('error', (error) => {
        console.error(`WebSocket error: ${error}`);
    });
});

// Log server start
console.log('WebSocket server running on ws://127.0.0.1:9001');

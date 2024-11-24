/**
 * WebSocket Server with JSON and CBOR Support
 *
 * This server listens for WebSocket connections on port 9001. It handles:
 * - JSON and CBOR messages from clients.
 * - Keep-alive pings and pong responses.
 * - Simulated server disconnect after a timeout.
 * - Periodic updates to connected clients.
 *
 * @module websocket_server
 */

const WebSocket = require('ws');
const cbor = require('cbor');

// Create a WebSocket server on port 9001
const wss = new WebSocket.Server({ port: 9001 });

/**
 * Handles new WebSocket client connections.
 *
 * Sets up event listeners for pings, messages, disconnections, and errors.
 * Responds to JSON and CBOR messages and sends periodic updates to the client.
 */
wss.on('connection', (ws) => {
    console.log('Client connected');

    /**
     * Handles ping messages from clients.
     *
     * Sends a pong response when a ping is received.
     */
    ws.on('ping', () => {
        console.log('Ping received, sending pong');
        ws.pong();
    });

    /**
     * Handles messages from the client.
     *
     * Supports both JSON and CBOR message formats. Sends a response based on the format of the received message.
     *
     * @param {Buffer|string} message - The raw message received from the client.
     */
    ws.on('message', (message) => {
        console.log(`Received raw message: ${message}`);

        // JSON Parsing
        try {
            const jsonMessage = JSON.parse(message);
            console.log('Parsed JSON:', jsonMessage);

            if (jsonMessage.type === 'greeting') {
                ws.send(
                    JSON.stringify({
                        type: 'response',
                        format: 'JSON',
                        content: 'Hello from server (JSON)!',
                    })
                );
                return;
            }
        } catch (e) {
            console.debug('Message is not JSON; trying CBOR.');
        }

        // CBOR Decoding
        try {
            const cborMessage = cbor.decodeFirstSync(message);
            console.log('Parsed CBOR:', cborMessage);

            if (cborMessage.type === 'greeting') {
                const response = cbor.encode({
                    type: 'response',
                    format: 'CBOR',
                    content: 'Hello from server (CBOR)!',
                });
                ws.send(response);
                return;
            }
        } catch (e) {
            console.debug('Message is not CBOR; trying JSON.');
        }

        // Unsupported message
        ws.send(`Unsupported format: ${message}`);
    });

    /**
     * Sends periodic updates to the client.
     *
     * This interval sends a JSON message every 5 seconds if the WebSocket is open.
     */
    const interval = setInterval(() => {
        if (ws.readyState === WebSocket.OPEN) {
            ws.send(
                JSON.stringify({
                    type: 'server_message',
                    content: 'Periodic update from server',
                })
            );
        }
    }, 5000);

    /**
     * Simulates a server-side disconnect after 30 seconds.
     *
     * Closes the WebSocket connection with a 1000 status code and a reason.
     */
    setTimeout(() => {
        if (ws.readyState === WebSocket.OPEN) {
            console.log('Simulating server disconnect');
            ws.close(1000, 'Simulated disconnect');
        }
    }, 30000);

    /**
     * Handles client disconnection events.
     *
     * Cleans up the periodic update interval to free resources.
     */
    ws.on('close', () => {
        console.log('Client disconnected');
        clearInterval(interval);
    });

    /**
     * Handles WebSocket errors.
     *
     * Logs error messages to the console.
     *
     * @param {Error} error - The error encountered by the WebSocket.
     */
    ws.on('error', (error) => {
        console.error(`WebSocket error: ${error}`);
    });
});

console.log('WebSocket server running on ws://127.0.0.1:9001');
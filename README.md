# WebSocket Toolkit

`WebSocket Toolkit` is a Rust crate designed to simplify WebSocket communication for real-time web applications. It provides the foundational structure to manage WebSocket connections, implement flexible reconnection strategies, handle multiple message formats with configurable serialization/deserialization, and maintain robust keep-alive mechanisms.

## Release # 1
This release serves as the initial framework for the project, setting up the core modules, dependency management, and basic features. Future versions will expand on this with more advanced functionality.

## Project Architecture

The project follows a modular design where each core feature (connection management, reconnection logic, message handling, keep-alive mechanism) is defined in its own module. Here's a breakdown of the project’s main components:

### Modules

1. **`connection.rs`**:
   - Manages WebSocket connections.
   - Implements the `WebSocketClient` struct, which provides basic connection setup and connection management.

2. **`reconnection.rs`**:
   - Provides a placeholder for reconnection logic with retries.
   - The `ReconnectStrategy` struct handles the number of retry attempts and will later implement reconnection behavior based on exponential backoff.

3. **`messages.rs`**:
   - Handles message serialization and deserialization.
   - Supports multiple formats including JSON and CBOR, using the `serde` library.

4. **`keep_alive.rs`**:
   - Implements the keep-alive mechanism to maintain WebSocket connections.
   - Uses `tokio::time` for periodic ping/pong frames, keeping the connection active.

5. **`lib.rs`**:
   - Acts as the central hub of the crate, re-exporting the modules and providing unit tests to ensure core functionality works correctly.

6. **`main.rs`**:
   - Provides an example executable that demonstrates how the library can be used in a real-world application.
   - It creates a `WebSocketClient` and connects to the provided WebSocket URL.

### Features:

- **Asynchronous, Non-Blocking Connection Management**:
  - The project uses the `tokio-tungstenite` crate along with Rust's async/await syntax to create efficient, non-blocking WebSocket connections.
  
- **Reconnection Logic (Placeholder)**:
  - The `ReconnectStrategy` provides the framework for retrying connections, allowing for future implementation of customizable reconnection strategies, such as exponential backoff.
  
- **Message Handling**:
  - Supports JSON and CBOR message formats using `serde`. The message handling module (`messages.rs`) allows for serialization and deserialization of messages, with flexibility for future formats.
  
- **Keep-Alive Mechanism**:
  - Periodically sends ping/pong frames to ensure that WebSocket connections remain active. The interval for pings is configurable.

## Crates Dependencies

The following dependencies are used in the project:

- **`tokio`**: Provides asynchronous runtime, enabling non-blocking operations.
  - Version: `1.0` (with `full` features).
  - Used for managing asynchronous tasks, such as maintaining WebSocket connections and sending/receiving messages.

- **`tokio-tungstenite`**: A WebSocket client/server library built on top of `tokio`.
  - Version: `0.15`.
  - Used to handle WebSocket connections asynchronously, enabling real-time communication.

- **`serde`**: A framework for serializing and deserializing data.
  - Version: `1.0` (with `derive` feature).
  - Used for flexible message serialization and deserialization in different formats (JSON, CBOR).

- **`serde_json`**: Provides JSON serialization/deserialization support.
  - Version: `1.0`.
  - Handles text-based WebSocket message formats.

- **`serde_cbor`**: Provides CBOR serialization/deserialization support.
  - Version: `0.11`.
  - Handles binary WebSocket message formats, useful for compact data exchanges.

- **`log`**: A logging library for Rust.
  - Version: `0.4`.
  - Provides logging functionality that will be used for debugging and tracking WebSocket events.

## Installation and Usage

1. **Clone the repository**:

   ```bash
   git clone https://github.com/SUMANTH571/Websocket-Toolkit.git
   cd websocket_toolkit
   ```

2. **Build the project**:

   ```bash
   cargo build
   ```

3. **Run the binary executable** (defined in `main.rs`):

   ```bash
   cargo run
   ```

   This will create a `WebSocketClient` and attempt to connect to the WebSocket server defined in the code.

4. **Run tests**:

   The project includes a basic test suite to verify the core functionality:

   ```bash
   cargo test
   ```





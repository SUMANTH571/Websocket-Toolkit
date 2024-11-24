# WebSocket Toolkit

`WebSocket Toolkit` is a Rust crate designed to simplify WebSocket communication for real-time web applications. It provides the foundational structure to manage WebSocket connections, implement flexible reconnection strategies, handle multiple message formats with configurable serialization/deserialization, and maintain robust keep-alive mechanisms.


## Release Notes:

### Release #1

This release serves as the initial framework for the project, setting up the core modules, dependency management, and basic features. Future versions will expand on this with more advanced functionality.

### Release #2

This release builds upon the initial framework by defining all public API functions, improving internal dependencies, and ensuring modularity. The project is now better structured to prepare for full functionality and documentation in future releases.

### Release #3

This release finalizes all core functionalities:
- **Full implementation** of connection management, message handling, reconnection strategies, and keep-alive mechanisms.
- **Example and testing** on how to develop applications using WebSocket Toolkit, with unit and integration testing.

### Final Release

The final release includes all features from previous releases and adds:
- **Comprehensive examples**: Includes detailed usage examples for connection setup, message handling, reconnection, and keep-alive.
- **Full documentation**: Using `cargo doc --open`, all modules, functions, and structs are thoroughly documented.
- **Unit and integration tests**: To ensure robustness and correctness.
- **Fuzz testing**: Validates deserialization against random data to prevent panics or crashes.
- **Docker Support**: Provides Docker and Docker Compose setup for seamless builds, testing, and example execution.
- **Crate Published on crate.io:** The crate has been published on the crates.io page by linking github repository.

## Project Architecture

The project follows a modular design where each core feature (connection management, reconnection logic, message handling, keep-alive mechanism) is defined in its own module. Here's a breakdown of the project’s main components:

### Modules

1. **`connection.rs`**:
   - Manages WebSocket connections.
   - Implements the `WebSocketClient` struct, which provides basic connection setup and connection management.

2. **`reconnection.rs`**:
   - Implements reconnection logic with retries and exponential backoff.
   - The `ReconnectStrategy` struct handles the number of retry attempts and delay configurations.

3. **`messages.rs`**:
   - Handles message serialization and deserialization.
   - Supports multiple formats, including JSON and CBOR, using the `serde` library.

4. **`keep_alive.rs`**:
   - Implements the keep-alive mechanism to maintain WebSocket connections.
   - Uses `tokio::time` for periodic ping/pong frames, keeping the connection active.

5. **`controller.rs`**:
   - Provides a high-level interface to manage WebSocket connections, integrating all other modules.

6. **`lib.rs`**:
   - Acts as the central hub of the crate, re-exporting the modules for user accessibility.
   - Contains unit tests for individual components.

7. **`main.rs`**:
   - Provides an example executable demonstrating how to use the library.
   - Creates a `WebSocketClient`, connects to a WebSocket server, and performs messaging tasks.


## Features

- **Asynchronous, Non-Blocking Connection Management**:
  - The project uses the `tokio-tungstenite` crate along with Rust's async/await syntax to create efficient, non-blocking WebSocket connections.
  
- **Reconnection Logic**:
  - The `ReconnectStrategy` provides customizable reconnection behavior with retries and exponential backoff.
  
- **Message Handling**:
  - Supports JSON and CBOR message formats using `serde`. The `messages` module handles serialization and deserialization, with flexibility for future formats.
  
- **Keep-Alive Mechanism**:
  - Periodically sends ping/pong frames to ensure that WebSocket connections remain active. The interval for pings is configurable.


## Crates Dependencies

### Core Dependencies

- **`tokio`**: Asynchronous runtime for non-blocking operations.
- **`tokio-tungstenite`**: WebSocket client/server library.
- **`serde`**: Serialization/deserialization framework.
- **`serde_json`**: JSON support.
- **`serde_cbor`**: CBOR support.
- **`log`**: Logging framework.

### Fuzzing

- **`cargo-fuzz`**: Fuzz testing framework for Rust.


## Installation and Usage

### Prerequisites

**1.  Install Rust (Stable and Nightly Toolchains)**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup install stable
   rustup install nightly
   rustup default stable
   ```
**2.  Install LLVM and Clang:**

- On Ubuntu:
   ```bash
   sudo apt-get update
   sudo apt-get install -y clang llvm libclang-dev
   ```
- On macOS:
   ```bash
   brew install llvm
   ```
**3. Install Cargo-Fuzz:**

```bash
cargo install cargo-fuzz
```

**Steps to use the crate:**
**1. Clone the repository:**

```bash
git clone https://github.com/SUMANTH571/Websocket-Toolkit.git
cd websocket_toolkit
```

**2. Build the project:**

```bash
cargo build
```

**3. Run tests:**

```bash
cargo test -- --nocapture
```
**Note:** If running locally, replace ws://node_server:9001 with ws://127.0.0.1:9001 in the tests.


**4. Run the example:**

```bash
cargo run --example simple_websocket
```

## Fuzz Testing:

**1.  Install cargo-fuzz:**
```bash
cargo install cargo-fuzz
```
**2.  Run fuzz testing:**
```bash
cargo fuzz run websocket_fuzz
```
**Note for Windows Users:** Fuzzing with cargo-fuzz may not work due to LLVM dependencies. Use a Linux VM, WSL, or the provided Docker environment for fuzzing.

## Docker Support & Setup
**1. Docker Compose File (docker-compose.yml):** Includes services for both the Rust application and a Node.js WebSocket server.

**2. Rust Dockerfile (Dockerfile.rust):** Installs Rust, cargo-fuzz, LLVM, and other dependencies.

**3. Node.js Dockerfile (Dockerfile.node):** Sets up a Node.js WebSocket server with required dependencies (ws and cbor).


## Build Documentation: 
Verify the crate’s documentation builds correctly.

```bash
cargo doc --open
```

## Publishing code to Crates.io:
This step requires API Token generated on crate.io accounts page to authenticate.

```bash
cargo login
```

Verify if the crate is ready to be published by running the following commands:

```bash
cargo check
cargo build
cargo test -- --nocapture 
```

Once all verifications are complete, publish the crate:

```bash
cargo publish
```

**The websocket crate we have built is now available at : https://crates.io/crates/websocket_toolkit/**

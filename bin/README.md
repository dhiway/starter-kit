# 🚦 Bin Module

This folder contains the main entry point for running the Starter Kit node as an application.

---

## What Does It Do?

- Parses command-line arguments (such as data path and secret key).
- Initializes the Iroh node, which manages networking, storage, and cryptographic identity.
- Starts the frontend (if enabled).
- Sets up the HTTP API server using Axum and the router module.
- Listens for incoming API requests on `http://localhost:4000`.
- Handles graceful shutdown on Ctrl+C.

---

## How to Run

1. **First-time setup:**  
   Generate a secret key by running:
   ```
   cargo run
   ```
   Save the secret key displayed in the output.

2. **Start the node:**  
   Use your data directory and secret key:
   ```
   cargo run -- --path <user_data_dir_path> --secret-key <secret_key>
   ```

3. **Access the API:**  
   The server will be available at [http://localhost:4000](http://localhost:4000).

---

## Main Components

- **`main.rs`**  
  - Entry point for the application.
  - Orchestrates node setup, frontend startup, and API server launch.
  - Handles shutdown signals gracefully.

---

## Related Files

- [`node/`](../node/) — Node initialization and management.
- [`router/`](../router/) — HTTP routing logic.
- [`helpers/`](../helpers/) — CLI parsing, state management, and utilities.

---

**Tip:**  
Check the terminal output for your Node ID and server status after startup.
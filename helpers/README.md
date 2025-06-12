# 🛠️ Helpers Module

This folder contains utility modules that support the core functionality of the Starter Kit.  
These helpers provide reusable logic for CLI parsing, frontend integration, application state management, and various utility functions.

---

## File Overview

### 1. `cli.rs`
Handles command-line argument parsing for the Starter Kit node.  
Defines the structure for CLI arguments (like data path and secret key) and provides parsing logic using Rust’s `clap` crate.  
**Why it matters:**  
Allows users to configure the node’s behavior at startup via command-line flags.

---

### 2. `frontend.rs`
Contains utilities for integrating with the frontend (such as the React app).  
May include CORS configuration, static file serving, or other helpers to bridge backend and frontend.  
**Why it matters:**  
Ensures smooth communication and resource sharing between the backend API and the frontend UI.

---

### 3. `state.rs`
Defines the `AppState` struct, which holds shared state for the application (such as references to the Iroh node, blobs, and docs clients).  
This state is injected into API handlers so they can access shared resources safely and efficiently.  
**Why it matters:**  
Centralizes application state, making it easy to manage and share across different parts of the backend.

---

### 4. `utils.rs`
A collection of utility functions and types used throughout the project.  
Includes:
- Encoding/decoding for document and author IDs
- Key validation and transformation helpers
- Download policy serialization/deserialization
- Other general-purpose helpers

**Why it matters:**  
Promotes code reuse and keeps business logic clean by moving common operations into dedicated functions.

---

## When to Use Helpers

- **CLI argument parsing:** Use `cli.rs`.
- **Frontend-backend integration:** Use `frontend.rs`.
- **Accessing shared state in handlers:** Use `state.rs`.
- **Common data transformations or validations:** Use `utils.rs`.

---

**Tip:**  
Keeping helpers modular and well-documented makes the codebase easier to maintain and extend.
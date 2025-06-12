# 🟢 Node Module

This folder contains the logic for initializing and managing the core Iroh node used by the Starter Kit backend.

---

## What is a Node?

In decentralized systems, a **node** is an instance that participates in the network—storing data, serving requests, and communicating with peers.  
Here, the node is built on top of [Iroh](https://github.com/n0-computer/iroh), providing persistent storage and networking for blobs and documents.

---

## Key Components

- **IrohNode Struct:**  
  Bundles the node’s identity, router, blobs, and docs clients for easy access throughout the application.

- **setup_iroh_node Function:**  
  Handles all steps to initialize a node:
  - Loads or generates a secret key for cryptographic identity.
  - Sets up persistent storage at the specified path.
  - Initializes Iroh’s networking, blobs, and docs subsystems.
  - Ensures the secret key is securely stored and verified.

---

## How to Use

1. **Generate a Secret Key:**  
   - On your first run, execute:
     ```
     cargo run
     ```
   - This will generate and display a new secret key for you.

2. **Start the Node:**  
   - Save the secret key from the previous step.
   - Start your node with:
     ```
     cargo run -- --path <user_data_dir_path> --secret-key <secret_key>
     ```
   - This will initialize the data store, set up the Iroh clients, and start the router for handling API requests.

---

## Why This Matters

- **Persistence:**  
  Your node’s identity and data are tied to the secret key and storage path.  
  Restarting with the same credentials restores your node’s state.

- **Security:**  
  Only someone with the secret key can operate your node.

- **Extensibility:**  
  The node exposes blobs and docs APIs, making it easy to build decentralized applications.

---

## Related Files

- [`helpers/cli.rs`](../helpers/cli.rs) — CLI argument parsing.
- [`core/`](../core/) — Business logic for blobs, docs, and authors.

---

**Tip:**  
Never share your secret key. It controls your node’s identity and access.
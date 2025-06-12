# 🚀 Starter Kit: Decentralized, Persistent Key-Value Document Store

Starter Kit is a developer-friendly framework for building decentralized, persistent, and cryptographically secure key-value document stores. Powered by [Iroh](https://github.com/n0-computer/iroh/tree/main), it provides robust primitives for blobs (raw data), docs (replicated key-value stores), and authors (identity and permissions), all with a simple API and persistent storage.

---

## ✨ Features

- **Decentralized Storage:** Peer-to-peer, content-addressed storage for documents and blobs.
- **Persistent Data:** All data is stored on disk; restart your node anytime with your secret key and data path.
- **Cryptographic Security:** Access and authorship are managed by keypairs; only you can start your node with your secret key.
- **Flexible Data Model:** Store any data as blobs, organize it in documents, and manage permissions with authors.
- **Extensible API:** JSON-over-HTTP API for easy integration with your apps.
- **Frontend Ready:** Comes with a React frontend for quick experimentation.

---

## 🏁 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+ recommended)
- [Node.js & npm](https://nodejs.org/) (for the frontend)

---

### 1. Clone and Build

```bash
git clone https://github.com/dhiway/starter-kit
cd starter-kit
cargo build --release
```

---

### 2. Bootstrap Your Node

#### **Step 1: Generate a Secret Key**

Run:

```bash
cargo run
```

You’ll see output like:

```
🔑 Generated new secret key: 9478a79a60ade2312eb50b0cc6f54444e8b81e9f06fa5df0b8d0b8ff5b1f60ab
👉 Please run again with:
   cargo run -- --path your-path-of-choice --secret-key 9478a79a60ade2312eb50b0cc6f54444e8b81e9f06fa5df0b8d0b8ff5b1f60ab
Error: "Rerun with --path and --secret-key"
```

> **Important:**  
> - **Never use a secret key from an example or from someone else.**
> - **Keep your secret key private and secure.**
> - **Anyone with your secret key can control your node and access your data.**

You can repeat this step until you get a secret key you like (not usually necessary).

#### **Step 2: Start Your Persistent Node**

Pick a directory for your data (e.g., `iroh-data`) and run:

```bash
cargo run -- --path iroh-data --secret-key <your-secret-key>
```

- The BLAKE3 hash of your secret key is stored in a `secret-key` file inside your data directory.
- **Security Note:** Only someone with your secret key can start your node and access your data.

---

### 3. Access the API and Frontend

- **Backend API:** Runs on [http://localhost:4000](http://localhost:4000)
- **Frontend:** The React app auto-starts on [http://localhost:3000](http://localhost:3000)

---

## 🗃️ Storage Model

- **Persistent:** All blobs and documents are stored on disk. Restart your node with the same `--path` and `--secret-key` to resume where you left off.
- **Blobs:** Store raw bytes, files, or any binary data.
- **Docs:** Key-value stores (replicas) with optional JSON schema enforcement.
- **Authors:** Manage multiple identities and permissions for document entries.

---

## 🧩 API Overview

The API exposes endpoints for:

- **Blobs:** Add, get, list, download, export, tag, and delete blobs.
- **Docs:** Create, list, share, join, drop, and manage documents.
- **Entries:** Add, get, query, and delete key-value entries (with or without schema).
- **Authors:** Create, list, set default, and verify authors.
- **Policies:** Set and get download policies for documents.

See the [API Documentation](./docs/) for full details and examples.

---

## 🛠️ Development

- **Run backend only:**  
  ```bash
  cargo run -- --path iroh-data --secret-key <your-secret-key>
  ```
- **Run frontend only:**  
  ```bash
  cd frontend
  npm install
  npm start
  ```

---

## 🧑‍💻 Contributing

Contributions are welcome! Fork the repo, create a branch, and submit a pull request.

---

## 🔒 Security Model

- Your node is protected by your secret key. The hash of this key is stored in your data directory; without the key, the node cannot be started.
- Each document and entry is cryptographically signed and content-addressed.
- Authors and namespaces are managed by keypairs for fine-grained access control.

---

## 📚 Learn More

- [Iroh Documentation](https://github.com/n0-computer/iroh/tree/main)
- [CORD](https://github.com/dhiway/cord)
- [API Reference](./docs/)

---

## 📄 License

TBD

---

**Starter Kit** — Build decentralized, persistent, and secure document stores with ease.
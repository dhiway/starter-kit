# 🛡️ Gateway Module

This folder contains the access control logic for the Starter Kit backend.  
The gateway acts as a security layer, ensuring that only authorized nodes and domains can access the HTTP APIs exposed by your node.

---

## What is the Gateway?

The **gateway** is responsible for managing and enforcing access control policies for all API endpoints.  
It maintains allow-lists for both **node IDs** (other nodes in the network) and **domains** (web origins), and checks every incoming request to ensure it is permitted.

Think of the gateway as a bouncer at the door:  
Only requests from trusted sources are allowed in.

---

## Key Features

- **Node ID Access Control:**  
  Only requests with an allowed `nodeId` header can access the APIs.  
  You can add or remove node IDs from the allow-list at runtime.

- **Domain Access Control:**  
  Requests from web browsers are checked against an allowed domain list (using the `Origin` header).  
  You can add or remove domains as needed.

- **Self-Access Guarantee:**  
  The node always allows itself access to its own APIs, so you never get locked out.

- **Persistent Storage:**  
  Allowed node IDs and domains are saved to disk and loaded on startup.

---

## How It Works

- The gateway maintains two allow-lists:
  - **Node IDs:** Other nodes that are permitted to call your APIs.
  - **Domains:** Web origins (e.g., `https://example.com`) that are allowed to access your APIs from a browser.

- Every incoming request is checked:
  - If it has a `nodeId` header, it must be in the allowed list.
  - If it has an `Origin` header, the domain must be allowed.
  - If neither is present, the request is rejected.

- You can manage these lists using the provided API endpoints.

---

## Example Scenario

Suppose you (the node owner) start a node.  
**Initially, only you will have access to the APIs.**  
As your needs grow, you can grant access to others by adding their node IDs or domain names.

- To add a new node ID, use the `/gateway/add-node-id` endpoint (handled by `add_node_id_handler`).
- To add a new domain, use the `/gateway/add-domain` endpoint (handled by `add_domain_handler`).

Only the node IDs and domains that have been added will have access to your APIs.  
This ensures that no unauthorized entity can use your APIs.

You can also remove node IDs and domains, or check if a particular node ID or domain is currently allowed.

---

## Why Use the Gateway?

- **Security:** Prevent unauthorized access to your node’s APIs.
- **Flexibility:** Dynamically manage who can connect, without restarting your node.
- **Persistence:** All changes are saved and restored on restart.
- **Peace of Mind:** You always retain access to your own APIs.

---

## Related Files

- [`access_control.rs`](./src/access_control.rs) — Core access control logic.
- [`storage.rs`](./src/storage.rs) — Persistent storage for allow-lists.
- [`api/`](../api/) — API handlers that use the gateway for access checks.

---

**Tip:**  
If you’re building a distributed or public-facing application, always configure your gateway rules to match your security needs!
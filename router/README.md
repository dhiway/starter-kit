# 🛣️ Router Module

This folder contains the routing logic for the Starter Kit backend.  
The router is responsible for mapping incoming HTTP requests to the appropriate handler functions in the API layer.

---

## What is a Router?

In web frameworks like Axum, a **router** defines which function should handle a given HTTP request based on its URL path and HTTP method (GET, POST, etc.).  
Think of it as a traffic controller that directs requests to the right place in your code.

---

## How It Works Here

- The main router is defined in [`router.rs`](./router.rs).
- It imports handler functions from the `api` module (`authors_handler`, `blobs_handler`, `docs_handler`).
- Each `.route()` call connects a URL path and HTTP method to a handler function.
- The router is initialized with application state, so handlers can access shared resources.

**Example:**
```rust
.route("/blobs/add-blob-bytes", post(add_blob_bytes_handler))
```
This means:  
A `POST` request to `/blobs/add-blob-bytes` will be handled by the `add_blob_bytes_handler` function.

---

## Why Separate Routing?

- **Organization:** Keeps routing logic separate from business logic.
- **Maintainability:** Easy to see all available endpoints in one place.
- **Scalability:** Add, remove, or change endpoints without touching handler code.

---

## Adding a New Route

1. Implement your handler function in the appropriate API handler file.
2. Import the handler in `router.rs`.
3. Add a new `.route()` line with the desired path and method.

---

## Related Files

- [`api/`](../api/) — Contains the actual handler implementations.
- [`core/`](../core/) — Business logic and data management.

---

**Tip:**  
If you’re new to Axum or Rust web servers, check out the [Axum routing documentation](https://docs.rs/axum/latest/axum/routing/index.html) for more details.
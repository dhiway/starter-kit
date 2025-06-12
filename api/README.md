# 🌐 API Module

This folder contains the HTTP API handler functions for the Starter Kit backend.  
Each file here (`authors_handler.rs`, `blobs_handler.rs`, `docs_handler.rs`) acts as a bridge between the core business logic and the web server, exposing the main features of the system as RESTful endpoints.

---

## Structure

- **`authors_handler.rs`**  
  Handlers for author management: create, list, set default, verify, and delete authors.

- **`blobs_handler.rs`**  
  Handlers for blob storage: add, retrieve, list, tag, download, and export blobs.

- **`docs_handler.rs`**  
  Handlers for document management: create, share, join, add entries, enforce schemas, and more.

---

## How It Works

- Each handler function receives HTTP requests, validates input, calls the appropriate core logic, and returns a structured HTTP response.
- Request and response types are defined for each endpoint, ensuring clear API contracts.
- Error handling is consistent, with clear status codes and messages for clients.

---

## API Reference

For detailed documentation of all available endpoints, request/response formats, and error handling,  
**please see the [API documentation in the `/docs` folder](../docs/)**:

- [Authors API](../docs/api/authors-api.md)
- [Blobs API](../docs/api/blobs-api.md)
- [Documents API](../docs/api/docs-api.md)

---

**Tip:**  
If you want to extend or customize the API, start by adding or modifying handler functions here, and update the documentation accordingly.
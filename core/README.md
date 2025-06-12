# 🧩 Core Module

This folder contains the core logic and abstractions for the Starter Kit, built on top of [Iroh](https://github.com/n0-computer/iroh).  
It provides the main building blocks for decentralized, persistent, and cryptographically secure document storage.

---

## Key Concepts

### **Author**
An **Author** represents an identity in the system, backed by a cryptographic keypair.  
Authors are used to sign and own entries in documents, and manage permissions.

**Example:**  
- Alice creates an author and uses it to add entries to a document.
- Each entry is signed by Alice’s author ID, ensuring authenticity.

---

### **Blob**
A **Blob** is a chunk of raw binary data, stored and addressed by its content hash.  
Blobs are useful for storing files, images, or any arbitrary data.

**Example:**  
- Upload a PDF as a blob.
- Retrieve the blob later using its hash, or tag it with a human-readable name.

---

### **Document (Doc)**
A **Document** is a decentralized, replicated key-value store.  
Documents can have schemas (for validation), support multiple authors, and store both structured data and file references.

**Example:**  
- Create a document for a digital certificate.
- Add entries like `owner`, `issue_date`, and attach a PDF as a blob entry.
- Enforce a schema so all entries follow a specific structure.

---

## File Structure

- [`authors.rs`](./src/authors.rs) — Author management logic.
- [`blobs.rs`](./src/blobs.rs) — Blob storage and retrieval logic.
- [`docs.rs`](./src/docs.rs) — Document creation, entry management, and schema enforcement.

---

## Function Reference

### 📇 Authors (`authors.rs`)

- **list_authors**: List all registered authors (returns SS58-encoded IDs).
- **get_default_author**: Get the current default author’s ID.
- **set_default_author**: Set a specific author as the default.
- **create_author**: Generate a new author and return its ID.
- **delete_author**: Remove an author by ID.
- **verify_author**: Check if an author exists.

**Error Handling:**  
Custom `AuthorError` enum covers all error cases (e.g., not found, invalid format, backend errors).

---

### 📦 Blobs (`blobs.rs`)

- **add_blob_bytes**: Store raw bytes as a blob.
- **add_blob_named**: Store bytes as a blob and assign a custom tag.
- **add_blob_from_path**: Import a file from disk as a blob.
- **list_blobs**: List blobs with pagination support.
- **get_blob**: Retrieve blob content by hash (as UTF-8 or base64).
- **status_blob**: Get the status of a blob (NotFound, Partial, Complete).
- **has_blob**: Check if a blob exists locally.
- **download_blob**: Download a blob from another node.
- **download_hash_sequence**: Download a sequence of blobs from a node.
- **download_with_options**: Download a blob with custom options (format, nodes, mode, etc.).
- **list_tags**: List all tags assigned to blobs.
- **delete_tag**: Remove a tag from the store.
- **export_blob_to_file**: Export a blob to a file on disk.

**Error Handling:**  
Custom `BlobError` enum for all blob-related errors.

---

### 📑 Documents (`docs.rs`)

- **get_document**: Open a document by its ID.
- **get_blob_entry**: Read and decode a blob entry from storage.
- **create_doc**: Create a new document and return its encoded ID.
- **list_docs**: List all documents and their capability types.
- **drop_doc**: Delete a document by its ID.
- **share_doc**: Generate a share ticket for a document.
- **join_doc**: Join a document using a share ticket.
- **close_doc**: Close an open document.
- **add_doc_schema**: Add a JSON schema to a document (enforces structure for future entries).
- **set_entry**: Add a key-value entry to a document (validates against schema if present).
- **set_entry_file**: Add a file as an entry (only if no schema is set).
- **get_entry**: Fetch an entry and its metadata.
- **get_entry_blob**: Retrieve the content of a blob entry by hash.
- **get_entries**: Query multiple entries with filters, pagination, and sorting.
- **delete_entry**: Remove an entry from a document.
- **leave**: Leave a document, releasing resources.
- **status**: Get the open status of a document.
- **get_download_policy**: Retrieve the download policy for a document.
- **set_download_policy**: Set or update the download policy for a document.

**Error Handling:**  
Custom `DocError` enum for all document-related errors.

---

## Documentation Structure

Because `docs.rs` and `blobs.rs` are particularly large and feature-rich, we have provided more detailed documentation for each in separate files (i.e., `docs/concepts/docs.md`, `docs/concepts/blobs.md`).  
However, this README serves as a concise reference and entry point for all core logic.

For detailed API usage and examples, see the [API documentation](../docs/).

---

**Tip:**  
If you’re unsure about a function or error, check the inline Rustdoc comments in each file for more details and usage examples.
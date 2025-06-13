# 📄 Docs in Iroh

In the Iroh ecosystem, a **Doc** (short for "document") is a decentralized, multi-dimensional key-value store designed for efficient synchronization and collaboration across peers.

---

## Key Concepts

### Key-Value Pairs

At its core, a Doc is a collection of **entries**, each of which is a key-value pair.  
- **Key:** A string that identifies the entry (e.g., `"owner"`, `"title"`, `"email"`).
- **Value:** The data associated with the key (e.g., `"Alice"`, `"My Document"`, `"alice@example.com"`).

**Example:**
```json
{
  "owner": "Alice",
  "title": "My Document",
  "created_at": "2025-06-13"
}
```

---

### Multi-Author and Multi-Replica

- **Authors:** Each entry is signed by an **author** (a cryptographic identity). Multiple authors can contribute to the same document.
- **Namespace:** Each document (or "replica") is identified by a unique namespace key. This key also controls write access.

---

### Entries and Content Addressing

Each entry in a Doc stores:
- The **key** (e.g., `"profile_picture"`).
- The **author** who wrote it.
- The **namespace** (the document it belongs to).
- The **value** is not stored directly—instead, the value is stored as a **BLAKE3 hash** of the content, along with the content size and a timestamp.

This means:
- The actual data (the value) is stored as a **blob** elsewhere (see [Blobs](./blobs.md)).
- The Doc only stores the hash and metadata, making it lightweight and efficient to sync.

**Example:**
- To store a large file (like a PDF), you first add it as a blob, then store its hash as the value for a key in the Doc.

---

### Schemas

Docs can optionally enforce a **schema** (using [JSON Schema](https://json-schema.org/)):
- A schema defines the structure and types of entries allowed in the document.
- **Important:** You can only add a schema to a document if it does not already contain any entries.  
  This is to prevent mismatches between existing entries and the schema.  
  If you wish to enforce a schema, always add it **before** adding any entries.
- Once a schema is set, all new entries must conform to it.
- This is useful for ensuring data consistency (e.g., requiring `"owner"` to always be a string).

---

### Synchronization

Docs are designed for efficient peer-to-peer synchronization:
- Peers exchange messages to reconcile their sets of entries.
- The protocol uses **range-based set reconciliation** to quickly find and sync differences, even in large documents.

---

### Storage

- Docs can be stored **in-memory** (for testing) or **persistently** (using an embedded database like redb).
- All entries, schemas, and metadata are stored in a single file for persistence.

---

## Why Use Docs?

- **Decentralized:** No central server; any peer can own and sync documents.
- **Multi-author:** Supports collaborative editing and audit trails.
- **Efficient:** Only hashes and metadata are synced; large data is handled via blobs.
- **Flexible:** Use with or without schemas, and store any kind of structured data.

---

## Example Use Cases

- **Digital Certificates:** Store certificate metadata as entries, attach the actual certificate file as a blob.
- **Collaborative Documents:** Multiple users add or update entries, with each change signed by its author.
- **Decentralized Profiles:** Store user information, profile pictures (as blobs), and settings in a single Doc.

---

## Further Reading

- [iroh-docs Documentation](https://docs.rs/iroh-docs/latest/iroh_docs/)
- [iroh-docs GitHub Repository](https://github.com/n0-computer/iroh-docs)

---

**In summary:**  
Docs in Iroh are decentralized, multi-author, schema-enforced key-value stores that efficiently synchronize across peers and reference large data via blobs. They are the backbone for building collaborative, persistent, and verifiable applications in the Iroh ecosystem.
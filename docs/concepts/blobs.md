# 🟣 Blobs in Iroh

This document explains what "blobs" are in the context of Iroh, how they are stored and transferred, and why the iroh-blobs protocol is designed the way it is.  
If you're new to decentralized storage or want to understand how Iroh handles raw data, this is the place to start.

---

## What is a Blob?

A **blob** is simply a sequence of bytes of arbitrary size, without any metadata.  
Blobs can represent files, images, documents, or any other binary data. In Iroh, blobs are the fundamental unit for storing and transferring raw data.

**Example:**  
- A PDF file, an image, or a chunk of application data can all be stored as blobs.

---

## Content Addressing with BLAKE3

Every blob in Iroh is identified by a **BLAKE3 hash** of its content.  
This means:
- The same data always has the same hash.
- You can verify the integrity of a blob by hashing its contents and comparing to its hash.
- Blobs are immutable: changing the data changes the hash.

**Link:**  
A 32-byte BLAKE3 hash that uniquely identifies a blob.

---

## The iroh-blobs Protocol

The [iroh-blobs](https://github.com/n0-computer/iroh-blobs) protocol enables efficient, verifiable transfer of blobs between devices.  
It is built on top of QUIC streams and uses BLAKE3-verified streaming to ensure data integrity.

### How it Works

- **Requester**: The side that asks for data (initiates requests).
- **Provider**: The side that serves data (answers requests).

A requester opens a QUIC stream to a provider and sends a request describing the desired data (by hash and byte range).  
The provider responds with the requested data, encoded as a BLAKE3-verified stream.  
This allows for:
- **Partial transfers** (requesting only a range of a blob)
- **Resumable transfers** (continue where you left off)
- **Integrity verification** (detect errors after at most 16 KiB)

---

## Blob Store Design

The **blob store** is responsible for storing blobs and their associated metadata.  
Iroh supports several storage backends:

- **In-memory store**: Fast, but not persistent. Good for testing or small data.
- **File system store**: Stores blobs as files on disk, using the hash as the filename.
- **Hybrid store**: Small blobs are stored inline in a database, large blobs as files. This provides good performance for both small and large blobs.

### Why Hybrid?

- **Databases** are efficient for many small blobs, but slow for large files.
- **File systems** are fast for large files, but inefficient for millions of tiny blobs.
- The hybrid approach uses the best of both:  
  - Small blobs (≤ 16 KiB) are stored in the database.
  - Large blobs are stored as files, with metadata in the database.

---

## Outboards and Metadata

For each blob, the store keeps:
- **Data**: The actual bytes of the blob.
- **Outboard**: A flattened hash tree (used for fast verification and partial reads).
- **Metadata**: Information about completeness, size, and tags.

Outboards are stored separately from data to allow for efficient verification and streaming.

---

## Blob Lifecycle

### Adding Local Files

- When you add a file, Iroh computes its BLAKE3 hash and outboard.
- The file is moved into the store under its hash.
- Data and outboard are stored either in the database or as files, depending on size.

### Syncing Remote Blobs

- When downloading from another node, data is written incrementally as chunks arrive.
- The store tracks which chunks are present and verified.
- You can start sharing a blob before the download is complete; only verified data is served.

### Deletion

- Blobs are protected from deletion by tags.
- Temporary tags prevent deletion while the process is running.
- Persistent tags keep blobs even after restart.
- Blobs can be explicitly deleted by hash (for emergencies).

---

## Advanced Concepts

### HashSeq

A **HashSeq** is a blob that contains a sequence of links (hashes).  
Useful for representing collections or ordered sets of blobs.

### Tags

Tags are human-readable names or labels you can assign to blobs for easier management.

---

## Performance and Reliability

- **Write batching**: Metadata updates are batched for performance, with a tradeoff in durability (acceptable for most use cases).
- **Partial entries**: The store efficiently handles incomplete blobs and can resume interrupted downloads.
- **Platform compatibility**: The hybrid store design works well across different operating systems and file systems.

---

## Further Reading

- [iroh-blobs Documentation](https://docs.rs/iroh-blobs/latest/iroh_blobs/)
- [iroh-blobs GitHub Repository](https://github.com/n0-computer/iroh-blobs)
- [Blob store design challenges (blog post)](https://blog.n0.computer/blob-store-design-challenges)

---

**In summary:**  
Blobs in Iroh are immutable, content-addressed chunks of data, efficiently stored and transferred using the iroh-blobs protocol.  
The hybrid blob store design ensures high performance for both small and large blobs, while BLAKE3 hashing and verified streaming guarantee data integrity and resumability.
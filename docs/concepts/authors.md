# 🧑‍💼 Authors in Iroh

In the Iroh ecosystem, an **Author** represents a unique cryptographic identity that can create, sign, and own entries in documents (Docs).  
Authors are fundamental for tracking who made which changes, enabling multi-user collaboration, and enforcing permissions in decentralized applications.

---

## What is an Author?

An Author in Iroh is:
- A cryptographic keypair (public/private keys).
- Identified by a public key, encoded as an SS58 address (a human-readable string).
- Used to sign entries in Docs, proving authorship and enabling audit trails.

---

## Why Are Authors Important?

- **Provenance:** Every entry in a Doc is signed by an Author, so you always know who created or modified data.
- **Collaboration:** Multiple Authors can contribute to the same Doc, supporting multi-user and multi-device workflows.
- **Permissions:** The Doc protocol can distinguish between Authors for access control or custom application logic.

---

## How Are Authors Used? (Typical Workflow)

1. **Doc Creation and Default Author:**  
   When a user creates a new Doc, they are automatically set as the default author for that document. Only this author can add entries initially.

2. **Adding More Authors:**  
   The default author can add additional authors to the document, enabling collaboration. Each added author gains permission to make changes to the Doc.

3. **Making Entries and Tracking Changes:**  
   Any authorized author can add or modify entries in the Doc. Each change is cryptographically signed by the author who made it, making it easy to trace every modification back to its origin.

4. **Listing and Verifying Authors:**  
   You can list all authors associated with a document and verify whether a given address is an author for that document. This helps manage permissions and audit contributions.

---

## Example: Multi-User Document

Suppose Alice creates a Doc. She is set as the default author and can add entries.  
Later, she adds Bob as an author so they can collaborate:

- Alice adds an entry:  
  `"task": "Design logo"` (signed by Alice’s Author)
- Bob adds an entry:  
  `"task": "Write documentation"` (signed by Bob’s Author)

When viewing the Doc, you can see which Author contributed each entry, and even filter or audit by Author.

---

## Author Lifecycle

- **Creation:** Authors are created and stored locally on your node.
- **Default Author:** The creator of a Doc is set as the default author.
- **Adding Authors:** More authors can be added to a Doc for collaboration.
- **Deletion:** Authors can be deleted if no longer needed.
- **Verification:** You can check if an Author exists and is associated with a Doc.

---

## Security Note

- **Private keys** for Authors are stored on your node.  
  Never share your private key; it controls your Author’s identity and signing power.

---

## Further Reading

- [SS58 Address Format (Polkadot Wiki)](https://wiki.polkadot.network/docs/learn-accounts#ss58-address-format)
- [iroh-docs Documentation](https://docs.rs/iroh-docs/latest/iroh_docs/)

---

**In summary:**  
Authors in Iroh are cryptographic identities that sign and own entries in Docs, enabling secure, auditable, and collaborative data management in decentralized applications.
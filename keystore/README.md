# 🔑 Keystore Module

This folder contains the logic for secure key management in the Starter Kit.  
The keystore is responsible for generating, storing, and retrieving cryptographic keypairs used for CORD and StarterKit operations.

---

## What is the Keystore?

The **keystore** is a secure, persistent storage for cryptographic keys used by the Starter Kit.  
It manages the **CORD** and **StarterKit** keypairs, which are essential for cryptographic operations and protocol-level identity.

**Important:**  
- **No private keys are ever stored on disk.**  
  Instead, private keys are generated at runtime using the provided SURI (Secret URI) and the corresponding public key.  
  This ensures that your private keys are never lost or leaked, and are only available in memory during operation.

---

## Key Features

- **Password Protection (`--password`):**  
  The keystore is always protected by a password.  
  This password is mandatory and required both for bootstrapping and restarting the node.

- **Optional Secret Encryption (`--secret`):**  
  You can provide an additional secret to encrypt the information stored in the keystore.  
  This is optional but recommended for extra security.  
  If you bootstrap the node with a secret, you must not provide the same secret on restart, it is just for encryption use.

- **Bootstrap & Open:**  
  - **Bootstrap:** Initialize a new keystore with a password (mandatory) and an optional secret for encryption.
  - **Open:** Load an existing keystore, requiring the correct password and, if used, the correct secret for decryption.

- **Keypair Management:**  
  - Generate and store **CORD** and **StarterKit** keypairs from a SURI.
  - Retrieve public and private keys for cryptographic operations.
  - Private keys are **never stored**; they are deterministically derived at runtime from the SURI and public key.

- **Integration:**  
  Used by the node initialization logic to ensure secure and persistent cryptographic identity across restarts.

---

## How It Works

- The keystore is implemented using Substrate's `LocalKeystore` for robust, cross-platform key management.
- Keys are stored on disk at a user-specified path, but **only public keys and metadata** are persisted.
- The keystore is always protected by a password, and can be further encrypted with an optional secret.
- Keypairs are generated from a SURI and inserted into the keystore under specific key types (`cord` and `starterkit`).
- Public keys can be listed and retrieved for cryptographic operations.
- **Private keys are always derived at runtime** using the SURI and public key, never written to disk.

---

## Security Note: The Importance of SURI

The **SURI (Secret URI)** is the most critical piece of information for your node's cryptographic identity.  
- It is used to generate both the CORD and StarterKit keypairs.
- **Never share your SURI with anyone.**
- Store your SURI in a secure location (such as a password manager or hardware security module).
- Losing your SURI means losing access to your cryptographic identity; leaking it means anyone can impersonate your node.

---

## Related Files

- [`node/`](../node/) — Node initialization and management, uses the keystore for CORD and StarterKit keypairs.
- [`helpers/cli.rs`](../helpers/cli.rs) — CLI argument parsing for keystore path, password, secret, and SURI.
- [`core/`](../core/) — Business logic that relies on cryptographic operations.

---

**Tip:**  
- Your password protects the keystore file, but your **SURI is the true secret**.  
  Guard it carefully—if you lose it, you lose your cryptographic identity; if it is leaked, your node can be compromised.
- No private key is ever stored on disk—your security is maximized by design.
- For maximum security, use both a strong password and a secret when bootstrapping your node.
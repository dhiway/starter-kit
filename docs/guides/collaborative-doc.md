# Guide: Creating a Collaborative Document

> **Note:**  
> Before calling these APIs, you must run  
> ```
> cargo run
> ```
> to generate your secret key, and then start the storage node with  
> ```
> cargo run -- --path <path_to_data_dir> --secret-key <secret_key>
> ```
> Only then will the APIs be available.

---

## Scenario

You want to create a document that multiple users can edit.

---

## Steps

### 1. Create a Document

- **API:** `POST /docs/create-document`
- **Why:** Initializes a new document. The creator is set as the default author.

### 2. Add Additional Authors

- **API:** `POST /authors/create-author` (on each collaborator's node)
- **Why:** Each collaborator creates their own author identity.

- **API:** (Custom, if supported) Add the new author's ID to the document's list of authors.
- **Why:** Grants edit permissions to other users.

### 3. Share the Document

- **API:** `POST /docs/share-doc`
- **Body:**  
  ```json
  {
    "doc_id": "<doc_id>",
    "mode": "Write",
    "addr_options": "Addresses"  # <- Look at documentation for options to add here
  }
  ```
- **Why:** Generates a ticket that collaborators can use to join the document.

### 4. Join the Document (Collaborator)

- **API:** `POST /docs/join-doc`
- **Body:** `{ "ticket": "<ticket>" }`
- **Why:** Allows a collaborator to join and edit the document.

### 5. Add Entries

- **API:** `POST /docs/set-entry`
- **Body:** 
  ```json
  {
    "doc_id": "<doc_id>",
    "author_id": "<author_id>",
    "key": "<key>",
    "value": "<value>"
  }
  ```
- **Why:** Each author can now add or update entries, and their changes are signed.

### 6. View Entries

- **API:** `POST /docs/get-entry`
- **Body:** 
  ```json
  {
    "doc_id": "<doc_id>",
    "author_id": "<author_id>", # <- of the author that added the entry
    "key": "<key>",
    "include_empty": bool
  }
  ```
- **Why:** Look at the different entries added by the authors(you need to know each author's ID).

---

## Result

Multiple users can now collaboratively edit the same document, with all changes tracked by author.
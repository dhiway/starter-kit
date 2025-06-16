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

---

### 2. Check the Default Author

- **API:** `GET /authors/get-default-author`
- **Why:** Returns the SS58-encoded author ID set as the default for your node.  
  **Response Example:**
  ```json
  {
    "default_author": "<author_id>"
  }
  ```
- Use this to confirm which author will be used for new entries.

---

### 3. Add Additional Authors

- **API:** `POST /authors/create-author`
- **Why:** Creates a new author identity (for a collaborator or another device).  
  **Response Example:**
  ```json
  {
    "author_id": "<new_author_id>"
  }
  ```
- Repeat this step for each collaborator you want to add.

---

### 4. Add Entries as Any Author

- All authors (default or newly created) can now add entries to the document using:
- **API:** `POST /docs/set-entry`
- **Body Example:**
  ```json
  {
    "doc_id": "<doc_id>",
    "author_id": "<author_id>",
    "key": "task",
    "value": "Write documentation"
  }
  ```
- **Why:** Each entry is signed by the author who adds it, enabling collaborative editing and traceability.

---

### 5. List All Authors for the Document

- **API:** `GET /authors/list-authors`
- **Why:** Returns a list of all SS58-encoded author IDs registered in the current context.  
  **Response Example:**
  ```json
  {
    "authors": [
      "<author_id_1>",
      "<author_id_2>",
      ...
    ]
  }
  ```
- Use this to see all collaborators who can edit the document.

---

## Result

Multiple users (authors) can now collaboratively edit the same document, with all changes tracked by author. You can always list and verify the authors associated with your document.
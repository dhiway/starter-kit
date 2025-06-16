# Guide: Enforcing a Schema on a Document

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

You want to ensure all entries in a document follow a specific structure (e.g., all entries must have an "owner" field as a string).

---

## Steps

### 1. Create a Document

- **API:** `POST /docs/create-document`
- **Why:** Start with a new, empty document.

### 2. Add a Schema

- **API:** `POST /docs/add-doc-schema`
- **Body:**  
  ```json
  {
    "author_id": "<your_author_id>",
    "doc_id": "<doc_id>",
    "schema": "{ \"type\": \"object\", \"properties\": { \"owner\": { \"type\": \"string\" } }, \"required\": [\"owner\"] }"
  }
  ```
- **Why:** Sets a JSON schema for the document.  
  **Note:** You must add the schema before adding any entries.

### 3. Add Entries

- **API:** `POST /docs/set-entry`
- **Body:** 
  ```json
  {
    "doc_id": "<doc_id>",
    "author_id": "<author_id>",
    "key": "<key>",
    "value": "{\"owner\": \"<owner>\"}"
  }
  ```
- **Why:** Now, only entries conforming to the schema can be added.

---

## Result

Your document now enforces a structure, ensuring data consistency.
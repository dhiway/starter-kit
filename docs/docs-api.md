# Documents API Documentation

This document describes the API endpoints and handler functions defined in `docs_handler.rs` and implemented in `docs.rs`. Each section details the request and response bodies, possible error responses, and success conditions.

---

## 1. Get Document

**Endpoint:**  
`POST /docs/get-document`

**Description:**  
Retrieves a document by its ID.

**Request Body:**
```json
{
  "doc_id": "string"
}
```
- `doc_id`: The document ID (base64-encoded string, required).

**Response:**

- **200 OK**
    ```json
    {
      "doc_id": "string",
      "status": "Document opened successfully"
    }
    ```
- **400 Bad Request**
    - `"doc_id cannot be empty"` if `doc_id` is missing.
    - `"Invalid doc_id: <error>"` if the ID format is invalid.
- **500 Internal Server Error**
    - `"DocumentNotFound"` or other error messages.

---

## 2. Get Blob Entry

**Endpoint:**  
`POST /docs/get-entry-blob`

**Description:**  
Reads and decodes a blob entry from storage by its hash.

**Request Body:**
```json
{
  "hash": "string"
}
```
- `hash`: The hash of the blob (required).

**Response:**

- **200 OK**
    ```json
    {
      "content": "string"
    }
    ```
- **400 Bad Request**
    - `"hash cannot be empty"` if `hash` is missing.
- **500 Internal Server Error**
    - `"FailedToReadBlob"`, `"FailedToConvertBlobUtf8"`, or other error messages.

---

## 3. Create Document

**Endpoint:**  
`POST /docs/create-document`

**Description:**  
Creates a new document and returns its encoded ID.

**Request Body:**  
_None._

**Response:**

- **200 OK**
    ```json
    {
      "doc_id": "string"
    }
    ```
- **500 Internal Server Error**
    - `"FailedToCreateDocument"` or other error messages.

---

## 4. List Documents

**Endpoint:**  
`GET /docs/list-docs`

**Description:**  
Lists all documents along with their capability types.

**Request Body:**  
_None._

**Response:**

- **200 OK**
    ```json
    [
      {
        "doc_id": "string",
        "capability": "Write"
      }
    ]
    ```
    - `capability`: `"Write"` or `"Read"`.

- **500 Internal Server Error**
    - `"FailedToListDocuments"` or other error messages.

---

## 5. Drop Document

**Endpoint:**  
`POST /docs/drop-doc`

**Description:**  
Deletes a document by its encoded ID.

**Request Body:**
```json
{
  "doc_id": "string"
}
```
- `doc_id`: The document ID to delete (required).

**Response:**

- **200 OK**
    ```json
    {
      "message": "Document dropped successfully"
    }
    ```
- **400 Bad Request**
    - `"doc_id cannot be empty"` if `doc_id` is missing.
- **500 Internal Server Error**
    - `"FailedToDropDocument"` or other error messages.

---

## 6. Share Document

**Endpoint:**  
`POST /docs/share-doc`

**Description:**  
Shares a document using the given mode and address options.

**Request Body:**
```json
{
  "doc_id": "string",
  "mode": "Read",
  "addr_options": "Addresses"
}
```
- `doc_id`: The document ID to share (required).
- `mode`: `"Read"` or `"Write"` (required).
- `addr_options`: `"Id"`, `"RelayAndAddresses"`, `"Relay"`, or `"Addresses"` (required).

**Response:**

- **200 OK**
    ```json
    {
      "ticket": "string"
    }
    ```
- **400 Bad Request**
    - `"doc_id cannot be empty"`, `"mode cannot be empty"`, `"addr_options cannot be empty"`.
    - `"Invalid share mode: <mode>"` or `"Invalid addr_options: <addr_options>"`.
- **500 Internal Server Error**
    - `"FailedToShareDocument"` or other error messages.

---

## 7. Join Document

**Endpoint:**  
`POST /docs/join-doc`

**Description:**  
Joins a shared document using its ticket.

**Request Body:**
```json
{
  "ticket": "string"
}
```
- `ticket`: The share ticket string (required).

**Response:**

- **200 OK**
    ```json
    {
      "doc_id": "string"
    }
    ```
- **400 Bad Request**
    - `"ticket cannot be empty"` if missing.
- **500 Internal Server Error**
    - `"InvalidDocumentTicketFormat"` or other error messages.

---

## 8. Close Document

**Endpoint:**  
`POST /docs/close-doc`

**Description:**  
Closes an open document.

**Request Body:**
```json
{
  "doc_id": "string"
}
```
- `doc_id`: The document ID to close (required).

**Response:**

- **200 OK**
    ```json
    {
      "message": "Document closed successfully"
    }
    ```
- **400 Bad Request**
    - `"doc_id cannot be empty"` if missing.
- **500 Internal Server Error**
    - `"FailedToCloseDocument"` or other error messages.

---

## 9. Add Document Schema

**Endpoint:**  
`POST /docs/add-doc-schema`

**Description:**  
Adds a JSON Schema to a document if it's currently empty.

**Request Body:**
```json
{
  "author_id": "string",
  "doc_id": "string",
  "schema": "{...}"
}
```
- `author_id`: SS58-encoded author ID (required).
- `doc_id`: Document ID (required).
- `schema`: JSON schema as a string (required).

**Response:**

- **200 OK**
    ```json
    {
      "updated_hash": "string"
    }
    ```
- **400 Bad Request**
    - `"author_id cannot be empty"`, `"doc_id cannot be empty"`, `"schema cannot be empty"`.
- **500 Internal Server Error**
    - `"FailedToSerializeSchema"`, `"FailedToValidateSchema"`, `"DocumentNotEmpty"`, or other error messages.

---

## 10. Set Entry

**Endpoint:**  
`POST /docs/set-entry`

**Description:**  
Adds a new entry (key-value pair) to the document after validating it against the schema, if one exists.

**Request Body:**
```json
{
  "doc_id": "string",
  "author_id": "string",
  "key": "string",
  "value": "string"
}
```
- `doc_id`: Document ID (required).
- `author_id`: SS58-encoded author ID (required).
- `key`: Key for the entry (required).
- `value`: Value as a JSON string (required).

**Response:**

- **200 OK**
    ```json
    {
      "hash": "string"
    }
    ```
- **400 Bad Request**
    - Any field missing or empty.
- **500 Internal Server Error**
    - `"FailedToValidateKey"`, `"ValueDoesNotMatchSchema"`, or other error messages.

---

## 11. Set Entry File

**Endpoint:**  
`POST /docs/set-entry-file`

**Description:**  
Adds a file as an entry to the document, only if no schema is defined.

**Request Body:**
```json
{
  "doc_id": "string",
  "author_id": "string",
  "key": "string",
  "file_path": "string"
}
```
- `doc_id`: Document ID (required).
- `author_id`: SS58-encoded author ID (required).
- `key`: Key for the entry (required).
- `file_path`: Path to the file (required).

**Response:**

- **200 OK**
    ```json
    {
      "key": "string",
      "hash": "string",
      "size": 123
    }
    ```
- **400 Bad Request**
    - Any field missing or empty.
- **500 Internal Server Error**
    - `"FileDoesNotExist"`, `"FileImportNotAllowedWithSchema"`, or other error messages.

---

## 12. Get Entry

**Endpoint:**  
`POST /docs/get-entry`

**Description:**  
Fetches an entry from a document along with metadata like hash and timestamp.

**Request Body:**
```json
{
  "doc_id": "string",
  "author_id": "string",
  "key": "string",
  "include_empty": true
}
```
- `doc_id`: Document ID (required).
- `author_id`: SS58-encoded author ID (required).
- `key`: Key to look up (required).
- `include_empty`: Boolean (optional).

**Response:**

- **200 OK**
    ```json
    {
      "doc": "string",
      "key": "string",
      "author": "string",
      "hash": "string",
      "len": 123,
      "timestamp": 123456789
    }
    ```
- **400 Bad Request**
    - Any field missing or empty.
- **404 Not Found**
    - `"Entry not found"` if the entry does not exist.
- **500 Internal Server Error**
    - `"FailedToGetEntry"` or other error messages.

---

## 13. Get Entries

**Endpoint:**  
`POST /docs/get-entries`

**Description:**  
Retrieves entries from a document based on provided query parameters.

**Request Body:**
```json
{
  "doc_id": "string",
  "query_params": "{...}"
}
```
- `doc_id`: Document ID (required).
- `query_params`: JSON string with optional fields:
    - `author_id`, `key`, `key_prefix`, `limit`, `offset`, `include_empty`, `sort_by`, `sort_direction`.

**Response:**

- **200 OK**
    ```json
    [
      {
        "doc": "string",
        "key": "string",
        "author": "string",
        "hash": "string",
        "len": 123,
        "timestamp": 123456789
      }
    ]
    ```
- **400 Bad Request**
    - `"doc_id cannot be empty"`, `"query_params cannot be empty"`, or invalid JSON.
- **500 Internal Server Error**
    - `"FailedToGetEntries"`, `"InvalidSortByValue"`, `"InvalidSortDirectionValue"`, or other error messages.

---

## 14. Delete Entry

**Endpoint:**  
`POST /docs/delete-entry`

**Description:**  
Deletes an entry from a document using author ID and key.

**Request Body:**
```json
{
  "doc_id": "string",
  "author_id": "string",
  "key": "string"
}
```
- `doc_id`: Document ID (required).
- `author_id`: SS58-encoded author ID (required).
- `key`: Key of the entry to delete (required).

**Response:**

- **200 OK**
    ```json
    {
      "deleted_count": 1
    }
    ```
- **400 Bad Request**
    - Any field missing or empty.
- **500 Internal Server Error**
    - `"EntryNotFound"` or other error messages.

---

## 15. Leave Document

**Endpoint:**  
`POST /docs/leave`

**Description:**  
Leaves the current document, releasing resources and closing its state.

**Request Body:**
```json
{
  "doc_id": "string"
}
```
- `doc_id`: Document ID (required).

**Response:**

- **200 OK**
    ```json
    {
      "message": "Successfully left document <doc_id>"
    }
    ```
- **400 Bad Request**
    - `"doc_id cannot be empty"` if missing.
- **500 Internal Server Error**
    - `"FailedToLeaveDocument"` or other error messages.

---

## 16. Status

**Endpoint:**  
`GET /docs/status`

**Description:**  
Retrieves the current open status of a document.

**Request Body:**
```json
{
  "doc_id": "string"
}
```
- `doc_id`: Document ID (required).

**Response:**

- **200 OK**
    ```json
    {
      "sync": true,
      "subscribers": 2,
      "handles": 1
    }
    ```
    - `sync`: Boolean indicating if the document is synchronized.
    - `subscribers`: Number of subscribers.
    - `handles`: Number of handles.

- **400 Bad Request**
    - `"doc_id cannot be empty"` if missing.
- **500 Internal Server Error**
    - `"FailedToGetDocumentStatus"` or other error messages.

---

## 17. Set Download Policy

**Endpoint:**  
`POST /docs/set-download-policy`

**Description:**  
Sets or updates the download policy of a document.

**Request Body:**
```json
{
  "doc_id": "string",
  "download_policy": "{...}"
}
```
- `doc_id`: Document ID (required).
- `download_policy`: JSON string representing the policy (required).

**Response:**

- **200 OK**
    ```json
    {
      "message": "Download policy set successfully"
    }
    ```
- **400 Bad Request**
    - `"doc_id cannot be empty"`, `"download_policy cannot be empty"`, or invalid JSON.
- **500 Internal Server Error**
    - `"FailedToSetDownloadPolicy"` or other error messages.

---

## 18. Get Download Policy

**Endpoint:**  
`GET /docs/get-download-policy`

**Description:**  
Fetches the download policy of a document, if any.

**Request Body:**
```json
{
  "doc_id": "string"
}
```
- `doc_id`: Document ID (required).

**Response:**

- **200 OK**
    ```json
    {
      "download_policy": "{...}"
    }
    ```
    - `download_policy`: JSON string representing the policy.

- **400 Bad Request**
    - `"doc_id cannot be empty"` if missing.
- **500 Internal Server Error**
    - `"FailedToGetDownloadPolicy"` or other error messages.

---

## Error Handling

- All endpoints return a `500 Internal Server Error` with a string message if an unexpected error occurs.
- For endpoints that require a request body, a `400 Bad Request` is returned if the required fields are missing or empty.
- On success, all endpoints return a `200 OK` status with the described response body.
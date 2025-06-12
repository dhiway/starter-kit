# Blobs API Documentation

This document describes the API endpoints and handler functions defined in `blobs_handler.rs`. Each section details the request and response bodies, possible error responses, and success conditions.

---

## 1. Add Blob Bytes

**Endpoint:**  
`POST /blobs/add-blob-bytes`

**Description:**  
Adds raw bytes as a blob.

**Request Body:**
```json
{
  "content": "string"
}
```
- `content`: The content to add as a blob (string, required).

**Response:**

- **200 OK**
    ```json
    {
      "hash": "string",
      "format": "Raw",
      "size": 123,
      "tag": "string"
    }
    ```
    - `hash`: The hash of the added blob.
    - `format`: The format of the blob (e.g., "Raw").
    - `size`: Size of the blob in bytes.
    - `tag`: Tag associated with the blob.

- **400 Bad Request**
    ```json
    "Content cannot be empty"
    ```
    - Returned if `content` is empty.

- **500 Internal Server Error**
    ```json
    "Failed to add blob: <error message>"
    ```
    - Returned if there is an internal error while adding the blob.

---

## 2. Add Blob Named

**Endpoint:**  
`POST /blobs/add-blob-named`

**Description:**  
Adds raw bytes as a blob and assigns a custom tag name.

**Request Body:**
```json
{
  "content": "string",
  "name": "string"
}
```
- `content`: The content to add as a blob (string, required).
- `name`: The custom tag name (string, required).

**Response:**

- **200 OK**
    ```json
    {
      "hash": "string",
      "format": "Raw",
      "size": 123,
      "tag": "string"
    }
    ```

- **400 Bad Request**
    - `"Content cannot be empty"` if `content` is empty.
    - `"Name cannot be empty"` if `name` is empty.

- **500 Internal Server Error**
    ```json
    "Failed to add named blob: <error message>"
    ```

---

## 3. Add Blob From Path

**Endpoint:**  
`POST /blobs/add-blob-from-path`

**Description:**  
Adds a file from the filesystem as a blob.

**Request Body:**
```json
{
  "file_path": "string"
}
```
- `file_path`: Path to the file (string, required).

**Response:**

- **200 OK**
    ```json
    {
      "hash": "string",
      "format": "Raw",
      "size": 123,
      "tag": "string"
    }
    ```

- **400 Bad Request**
    - `"File path cannot be empty"` if `file_path` is empty.
    - `"File does not exist"` if the file does not exist at the given path.

- **500 Internal Server Error**
    ```json
    "Failed to add blob from path: <error message>"
    ```

---

## 4. List Blobs

**Endpoint:**  
`POST /blobs/list-blobs`

**Description:**  
Lists blobs stored in the blob store with pagination.

**Request Body:**
```json
{
  "page": 0,
  "page_size": 10
}
```
- `page`: Page number (zero-based, required).
- `page_size`: Number of blobs per page (required, must be > 0).

**Response:**

- **200 OK**
    ```json
    [
      {
        "path": "string",
        "hash": "string",
        "size": 123
      }
    ]
    ```
    - Array of blob info objects.

- **400 Bad Request**
    - `"Page size must be greater than 0"` if `page_size` is zero.

- **500 Internal Server Error**
    ```json
    "Failed to list blobs: <error message>"
    ```

---

## 5. Get Blob

**Endpoint:**  
`POST /blobs/get-blob`

**Description:**  
Reads a blob's content by hash and returns it as a UTF-8 string or base64-encoded string if binary.

**Request Body:**
```json
{
  "hash": "string"
}
```
- `hash`: The hash identifying the blob (string, required).

**Response:**

- **200 OK**
    ```json
    {
      "content": "string"
    }
    ```
    - `content`: The blob content as a string.

- **400 Bad Request**
    - `"Hash cannot be empty"` if `hash` is empty.

- **500 Internal Server Error**
    ```json
    "Failed to get blob: <error message>"
    ```

---

## 6. Status Blob

**Endpoint:**  
`POST /blobs/status-blob`

**Description:**  
Gets the current status of a blob by its hash (e.g., NotFound, Partial, Complete).

**Request Body:**
```json
{
  "hash": "string"
}
```
- `hash`: The hash identifying the blob (string, required).

**Response:**

- **200 OK**
    ```json
    {
      "status": "Complete"
    }
    ```
    - `status`: Status of the blob (`"NotFound"`, `"Partial"`, or `"Complete"`).

- **400 Bad Request**
    - `"Hash cannot be empty"` if `hash` is empty.

- **500 Internal Server Error**
    ```json
    "Failed to get blob status: <error message>"
    ```

---

## 7. Has Blob

**Endpoint:**  
`POST /blobs/has-blob`

**Description:**  
Checks if a blob with the given hash exists locally.

**Request Body:**
```json
{
  "hash": "string"
}
```
- `hash`: The hash to check for existence (string, required).

**Response:**

- **200 OK**
    ```json
    {
      "present": true
    }
    ```
    - `present`: Boolean indicating if the blob exists.

- **400 Bad Request**
    - `"Hash cannot be empty"` if `hash` is empty.

- **500 Internal Server Error**
    ```json
    "Failed to check blob presence: <error message>"
    ```

---

## 8. Download Blob

**Endpoint:**  
`POST /blobs/download-blob`

**Description:**  
Downloads a blob from a specified node.

**Request Body:**
```json
{
  "hash": "string",
  "node_id": "string"
}
```
- `hash`: The hash of the blob to download (string, required).
- `node_id`: The node ID to download the blob from (string, required).

**Response:**

- **200 OK**
    ```json
    {
      "local_size": 123,
      "downloaded_size": 123,
      "stats": "string"
    }
    ```
    - `local_size`: Size of the blob locally.
    - `downloaded_size`: Size downloaded.
    - `stats`: Download statistics.

- **400 Bad Request**
    - `"Hash cannot be empty"` if `hash` is empty.
    - `"Node ID cannot be empty"` if `node_id` is empty.

- **500 Internal Server Error**
    ```json
    "Failed to download blob: <error message>"
    ```

---

## 9. Download Hash Sequence

**Endpoint:**  
`POST /blobs/download-hash-sequence`

**Description:**  
Downloads a sequence of hashes from a specified node.

**Request Body:**
```json
{
  "hash": "string",
  "node_id": "string"
}
```
- `hash`: The hash to download (string, required).
- `node_id`: The node ID to download from (string, required).

**Response:**

- **200 OK**
    ```json
    {
      "local_size": 123,
      "downloaded_size": 123,
      "stats": "string"
    }
    ```

- **400 Bad Request**
    - `"Hash cannot be empty"` if `hash` is empty.
    - `"Node ID cannot be empty"` if `node_id` is empty.

- **500 Internal Server Error**
    ```json
    "Failed to download hash sequence: <error message>"
    ```

---

## 10. Download With Options

**Endpoint:**  
`POST /blobs/download-with-options`

**Description:**  
Downloads a blob with custom download options.

**Request Body:**
```json
{
  "hash": "string",
  "format": "Raw",
  "mode": "Direct",
  "nodes": ["node_id1", "node_id2"],
  "tag": "Auto"
}
```
- `hash`: The hash of the blob to download (string, required).
- `format`: Download format (`"Raw"` or `"HashSeq"`, required).
- `mode`: Download mode (`"Direct"` or `"Queued"`, required).
- `nodes`: Array of node IDs (at least one required).
- `tag`: Tag option (`"Auto"` or a custom string, required).

**Response:**

- **200 OK**
    ```json
    {
      "local_size": 123,
      "downloaded_size": 123,
      "stats": "string"
    }
    ```

- **400 Bad Request**
    - `"Hash cannot be empty"` if `hash` is empty.
    - `"Format cannot be empty"` if `format` is empty.
    - `"Mode cannot be empty"` if `mode` is empty.
    - `"Nodes cannot be empty"` if `nodes` is empty.
    - `"Tag cannot be empty"` if `tag` is empty.
    - `"Invalid format: <format>"` if `format` is not recognized.
    - `"Invalid mode: <mode>"` if `mode` is not recognized.
    - `"Invalid node ID '<id>': <error>"` if any node ID is invalid.

- **500 Internal Server Error**
    ```json
    "<error message>"
    ```

---

## 11. List Tags

**Endpoint:**  
`GET /blobs/list-tags`

**Description:**  
Lists all available tags.

**Request Body:**  
_None._

**Response:**

- **200 OK**
    ```json
    [
      {
        "name": "string",
        "format": "Raw",
        "hash": "string"
      }
    ]
    ```
    - Array of tag info objects.

- **500 Internal Server Error**
    ```json
    "<error message>"
    ```

---

## 12. Delete Tag

**Endpoint:**  
`POST /blobs/delete-tag`

**Description:**  
Deletes a specific tag.

**Request Body:**
```json
{
  "tag_name": "string"
}
```
- `tag_name`: The name of the tag to delete (string, required).

**Response:**

- **200 OK**
    ```json
    {
      "message": "Tag deleted successfully"
    }
    ```

- **400 Bad Request**
    - `"Tag name cannot be empty"` if `tag_name` is empty.

- **500 Internal Server Error**
    ```json
    "<error message>"
    ```

---

## 13. Export Blob to File

**Endpoint:**  
`POST /blobs/export-blob-to-file`

**Description:**  
Exports a blob to a file on disk.

**Request Body:**
```json
{
  "hash": "string",
  "destination": "string"
}
```
- `hash`: The hash of the blob to export (string, required).
- `destination`: The file path where the blob should be saved (string, required).

**Response:**

- **200 OK**
    ```json
    {
      "message": "Blob <hash> exported to <destination>"
    }
    ```

- **400 Bad Request**
    - `"Hash cannot be empty"` if `hash` is empty.
    - `"Destination cannot be empty"` if `destination` is empty.

- **500 Internal Server Error**
    ```json
    "<error message>"
    ```

---

## Error Handling

- All endpoints return a `500 Internal Server Error` with a string message if an unexpected error occurs.
- For endpoints that require a request body, a `400 Bad Request` is returned if the required fields are missing or empty.
- On success, all endpoints return a `200 OK` status with the described response body.
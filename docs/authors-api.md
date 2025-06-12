# Authors API Documentation

This document describes the API endpoints and handler functions defined in `authors_handler.rs` and implemented in `authors.rs`. Each section details the request and response bodies, possible error responses, and success conditions.

---

## 1. List Authors

**Endpoint:**  
`GET /authors/list-authors`

**Description:**  
Returns a list of all author IDs registered in the current context.

**Request Body:**  
_None._

**Response:**

- **200 OK**
    ```json
    {
      "authors": ["author_id_1", "author_id_2", ...]
    }
    ```
    - `authors`: Array of SS58-encoded author IDs.

- **500 Internal Server Error**
    - `"FailedToListAuthors"`, `"StreamingError"`, `"InvalidAuthorIdFormat"`, or `"FailedToCollectAuthors"`.

---

## 2. Get Default Author

**Endpoint:**  
`GET /authors/get-default-author`

**Description:**  
Fetches the current default author ID.

**Request Body:**  
_None._

**Response:**

- **200 OK**
    ```json
    {
      "default_author": "author_id"
    }
    ```
    - `default_author`: The SS58-encoded ID of the default author.

- **500 Internal Server Error**
    - `"DefaultAuthorNotFound"` or `"InvalidAuthorIdFormat"`.

---

## 3. Set Default Author

**Endpoint:**  
`POST /authors/set-default-author`

**Description:**  
Sets the default author to the provided author ID.

**Request Body:**
```json
{
  "author_id": "string"
}
```
- `author_id`: The SS58-encoded ID of the author to set as default (required).

**Response:**

- **200 OK**
    ```json
    {
      "message": "Default author set successfully"
    }
    ```

- **400 Bad Request**
    - `"author_id cannot be empty"` if `author_id` is missing or empty.

- **500 Internal Server Error**
    - `"InvalidAuthorIdFormat"` or `"FailedToSetDefaultAuthor"`.

---

## 4. Create Author

**Endpoint:**  
`POST /authors/create-author`

**Description:**  
Creates a new author and returns its ID.

**Request Body:**  
_None._

**Response:**

- **200 OK**
    ```json
    {
      "author_id": "string"
    }
    ```
    - `author_id`: The SS58-encoded ID of the newly created author.

- **500 Internal Server Error**
    - `"FailedToCreateAuthor"` or `"InvalidAuthorIdFormat"`.

---

## 5. Delete Author

**Endpoint:**  
`POST /authors/delete-author`

**Description:**  
Deletes an author based on its ID.

**Request Body:**
```json
{
  "author_id": "string"
}
```
- `author_id`: The SS58-encoded ID of the author to delete (required).

**Response:**

- **200 OK**
    ```json
    {
      "message": "Author deleted successfully"
    }
    ```

- **400 Bad Request**
    - `"author_id cannot be empty"` if `author_id` is missing or empty.

- **500 Internal Server Error**
    - `"InvalidAuthorIdFormat"`, `"FailedToListAuthors"`, `"FailedToCollectAuthors"`, `"FailedToDeleteAuthor"`.
    - `"AuthorNotFound"` if the author does not exist.

---

## 6. Verify Author

**Endpoint:**  
`POST /authors/verify-author`

**Description:**  
Verifies whether a given author ID exists.

**Request Body:**
```json
{
  "author_id": "string"
}
```
- `author_id`: The SS58-encoded ID of the author to verify (required).

**Response:**

- **200 OK**
    ```json
    {
      "is_valid": true
    }
    ```
    - `is_valid`: Boolean indicating if the author exists.

- **400 Bad Request**
    - `"author_id cannot be empty"` if `author_id` is missing or empty.

- **500 Internal Server Error**
    - `"InvalidAuthorIdFormat"`, `"FailedToListAuthors"`, or `"FailedToCollectAuthors"`.

---

## Error Handling

- All endpoints return a `500 Internal Server Error` with a string message if an unexpected error occurs.
- For endpoints that require a request body, a `400 Bad Request` is returned if the required fields are missing or empty.
- On success, all endpoints return a `200 OK` status with the described response body.
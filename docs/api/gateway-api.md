# Gateway API Documentation

This document describes the API endpoints and handler functions defined in `gateway_handler.rs` and implemented in the gateway module.  
Each section details the endpoint, request and response bodies, possible error responses, and success conditions.

---

## 1. Check if a Node ID is Allowed

**Endpoint:**  
`GET /gateway/is-node-id-allowed`

**Description:**  
Checks whether a given node ID is currently allowed to access the APIs.

**Request Body:**  
```json
{
  "node_id": "string"
}
```
- `node_id`: The node ID to check (required).

**Response:**

- **200 OK**
    ```json
    {
      "allowed": true
    }
    ```
    - `allowed`: Boolean indicating if the node ID is allowed.

- **400 Bad Request**
    - `"nodeId cannot be empty"` if `node_id` is missing or empty.
    - `"nodeId is not a valid NodeId"` if the format is invalid.

---

## 2. Check if a Domain is Allowed

**Endpoint:**  
`GET /gateway/is-domain-allowed`

**Description:**  
Checks whether a given domain is currently allowed to access the APIs.

**Request Body:**  
```json
{
  "domain": "string"
}
```
- `domain`: The domain to check (required).

**Response:**

- **200 OK**
    ```json
    {
      "allowed": true
    }
    ```
    - `allowed`: Boolean indicating if the domain is allowed.

- **400 Bad Request**
    - `"domain cannot be empty"` if `domain` is missing or empty.
    - `"Invalid domain format"` if the domain does not match the expected pattern.

---

## 3. Add a Node ID

**Endpoint:**  
`POST /gateway/add-node-id`

**Description:**  
Adds a node ID to the allow-list, granting it access to the APIs.

**Request Body:**  
```json
{
  "node_id": "string"
}
```
- `node_id`: The node ID to add (required).

**Response:**

- **200 OK**
    ```json
    {
      "message": "Node ID added successfully"
    }
    ```

- **400 Bad Request**
    - `"nodeId cannot be empty"` if `node_id` is missing or empty.
    - `"nodeId is not a valid NodeId"` if the format is invalid.

---

## 4. Remove a Node ID

**Endpoint:**  
`POST /gateway/remove-node-id`

**Description:**  
Removes a node ID from the allow-list, revoking its access to the APIs.

**Request Body:**  
```json
{
  "node_id": "string"
}
```
- `node_id`: The node ID to remove (required).

**Response:**

- **200 OK**
    ```json
    {
      "message": "Node ID removed successfully"
    }
    ```

- **400 Bad Request**
    - `"nodeId cannot be empty"` if `node_id` is missing or empty.
    - `"nodeId is not a valid NodeId"` if the format is invalid.

---

## 5. Add a Domain

**Endpoint:**  
`POST /gateway/add-domain`

**Description:**  
Adds a domain to the allow-list, granting it access to the APIs.

**Request Body:**  
```json
{
  "domain": "string"
}
```
- `domain`: The domain to add (required).

**Response:**

- **200 OK**
    ```json
    {
      "message": "Domain added successfully"
    }
    ```

- **400 Bad Request**
    - `"domain cannot be empty"` if `domain` is missing or empty.
    - `"Invalid domain format"` if the domain does not match the expected pattern.

---

## 6. Remove a Domain

**Endpoint:**  
`POST /gateway/remove-domain`

**Description:**  
Removes a domain from the allow-list, revoking its access to the APIs.

**Request Body:**  
```json
{
  "domain": "string"
}
```
- `domain`: The domain to remove (required).

**Response:**

- **200 OK**
    ```json
    {
      "message": "Domain removed successfully"
    }
    ```

- **400 Bad Request**
    - `"domain cannot be empty"` if `domain` is missing or empty.
    - `"Invalid domain format"` if the domain does not match the expected pattern.

---

## Error Handling

- All endpoints return a `400 Bad Request` with a string message if required fields are missing, empty, or invalid.
- On success, all endpoints return a `200 OK` status with the described response body.
- If an unexpected error occurs, a `500 Internal Server Error` with a string message may be returned.

---
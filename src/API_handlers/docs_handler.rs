use crate::iroh_core::docs::{get_document, get_entry_blob, create_doc, list_docs, drop_doc, share_doc, join_doc, close_doc, add_doc_schema, set_entry, set_entry_file, get_entry, get_entries, delete_entry, leave, status, set_download_policy, get_download_policy};
use crate::helpers::state::AppState;
use serde::{Deserialize, Serialize};
use axum::{extract::State, Json};
use axum::http::StatusCode;
use std::str::FromStr;
use std::sync::Arc;
use iroh_docs::{NamespaceId, CapabilityKind};
use iroh_blobs::Hash;
use iroh_docs::rpc::client::docs::ShareMode;
use iroh_docs::rpc::AddrInfoOptions;

// Request bodies
// 1. get document
#[derive(Deserialize)]
pub struct GetDocumentRequest {
    pub doc_id: String,
}

// 2. get blob entry
#[derive(Deserialize)]
pub struct GetEntryBlobRequest {
    pub hash: String,
}

// 3. create document
// No request body

// 4. list docs
// No request body

// 5. drop doc
#[derive(Deserialize)]
pub struct DropDocRequest {
    pub doc_id: String,
}

// 6. share doc
#[derive(Deserialize)]
pub struct ShareDocRequest {
    pub doc_id: String,
    pub mode: String,
    pub addr_options: String,
}

// 7. join doc
#[derive(Deserialize)]
pub struct JoinDocRequest {
    pub ticket: String,
}

// 8. close document
#[derive(Deserialize)]
pub struct CloseDocRequest {
    pub doc_id: String,
}

// 9. add document schema
#[derive(Deserialize)]
pub struct AddDocSchemaRequest {
    pub author_id: String,
    pub doc_id: String,
    pub schema: String, // Should be a valid JSON string
}

// 10. set entry
#[derive(Debug, Deserialize)]
pub struct SetEntryRequest {
    pub doc_id: String,
    pub author_id: String,
    pub key: String,
    pub value: String,
}

// 11. set entry file
#[derive(Debug, Deserialize)]
pub struct SetEntryFileRequest {
    pub doc_id: String,
    pub author_id: String,
    pub key: String,
    pub file_path: String,
}

// 12. get entry
#[derive(Debug, Deserialize)]
pub struct GetEntryRequest {
    pub doc_id: String,
    pub author_id: String,
    pub key: String,
    pub include_empty: bool,
}

// 13. get entries
#[derive(Deserialize)]
pub struct GetEntriesRequest {
    pub doc_id: String,
    pub query_params: String, // JSON string from user
}

// 14. delete entry
#[derive(Deserialize)]
pub struct DeleteEntryRequest {
    pub doc_id: String,
    pub author_id: String,
    pub key: String,
}

// 15. leave document
#[derive(Deserialize)]
pub struct LeaveRequest {
    pub doc_id: String,
}

// 16. status
#[derive(Deserialize)]
pub struct StatusRequest {
    pub doc_id: String,
}

// 17. set download policy
#[derive(Deserialize)]
pub struct SetDownloadPolicyRequest {
    pub doc_id: String,
    pub download_policy: String, // JSON as string input
}

// 18. get download policy
#[derive(Deserialize)]
pub struct GetDownloadPolicyRequest {
    pub doc_id: String,
}

// Response bodies
// 1. get document
#[derive(Serialize)]
pub struct GetDocumentResponse {
    pub doc_id: String,
    pub status: String,
}

// 2. get blob entry
#[derive(Serialize)]
pub struct GetEntryBlobResponse {
    pub content: String,
}

// 3. create document
#[derive(Serialize)]
pub struct CreateDocResponse {
    pub doc_id: String,
}

// 4. list docs
#[derive(Serialize)]
pub struct ListDocsResponse {
    pub doc_id: String,
    pub capability: String,
}

// 5. drop doc
#[derive(Serialize)]
pub struct DropDocResponse {
    pub message: String,
}

// 6. share doc
#[derive(Serialize)]
pub struct ShareDocResponse {
    pub ticket: String,
}

// 7. join doc
#[derive(Serialize)]
pub struct JoinDocResponse {
    pub doc_id: String,
}

// 8. close document
#[derive(Serialize)]
pub struct CloseDocResponse {
    pub message: String,
}

// 9. add document schema
#[derive(Serialize)]
pub struct AddDocSchemaResponse {
    pub updated_hash: String,
}

// 10. set entry
#[derive(Debug, Serialize)]
pub struct SetEntryResponse {
    pub hash: String,
}

// 11. set entry file
#[derive(Debug, Serialize)]
pub struct SetEntryFileResponse {
    pub key: String,
    pub hash: String,
    pub size: u64,
}

// 12. get entry
#[derive(Debug, Serialize)]
pub struct GetEntryResponse {
    pub doc: String,
    pub key: String,
    pub author: String,
    pub hash: String,
    pub len: u64,
    pub timestamp: u64,
}

// 13. get entries
#[derive(Serialize)]
pub struct GetEntriesResponse {
    pub entries: Vec<GetEntryResponse>,
}

// 14. delete entry
#[derive(Serialize)]
pub struct DeleteEntryResponse {
    pub deleted_count: usize,
}

// 15. leave document
#[derive(Serialize)]
pub struct LeaveResponse {
    pub message: String,
}

// 16. status
#[derive(Serialize)]
pub struct StatusResponse {
    pub sync: bool,
    pub subscribers: usize,
    pub handles: usize,
}

// 17. set download policy
#[derive(Serialize)]
pub struct SetDownloadPolicyResponse {
    pub message: String,
}

// 18. get download policy
#[derive(Serialize)]
pub struct GetDownloadPolicyResponse {
    pub download_policy: String, // Return JSON as string
}

// Handler for getting a document
pub async fn get_document_handler(
    State(state): State<AppState>,
    Json(payload): Json<GetDocumentRequest>,
) -> Result<Json<GetDocumentResponse>, (StatusCode, String)> {
    let doc_id = NamespaceId::from_str(&payload.doc_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid doc_id: {}", e)))?;

    match get_document(state.docs.clone(), doc_id).await {
        Ok(doc) => Ok(Json(GetDocumentResponse {
            doc_id: doc.id().to_string(),
            status: "Document opened successfully".to_string(),
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }   
}

// Handler for getting a blob entry
pub async fn get_entry_blob_handler(
    State(state): State<AppState>,
    Json(payload): Json<GetEntryBlobRequest>,
) -> Result<Json<GetEntryBlobResponse>, (StatusCode, String)> {
    match get_entry_blob(state.blobs.clone(), payload.hash).await {
        Ok(content) => Ok(Json(GetEntryBlobResponse { content })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for creating a new document
pub async fn create_doc_handler(
    State(state): State<AppState>,
) -> Result<Json<CreateDocResponse>, (StatusCode, String)> {
    match create_doc(state.docs.clone()).await {
        Ok(doc_id) => Ok(Json(CreateDocResponse { doc_id })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for listing documents
pub async fn list_docs_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<ListDocsResponse>>, (StatusCode, String)> {
    match list_docs(state.docs.clone()).await {
        Ok(docs) => {
            let response = docs
                .into_iter()
                .map(|(doc_id, capability)| {
                    let capability_str = match capability {
                        CapabilityKind::Write => "Write".to_string(),
                        CapabilityKind::Read => "Read".to_string(),
                    };

                    ListDocsResponse {
                        doc_id,
                        capability: capability_str,
                    }
                })
                .collect();

            Ok(Json(response))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for dropping a document
pub async fn drop_doc_handler(
    State(state): State<AppState>,
    Json(payload): Json<DropDocRequest>,
) -> Result<Json<DropDocResponse>, (StatusCode, String)> {
    match drop_doc(state.docs.clone(), payload.doc_id).await {
        Ok(_) => Ok(Json(DropDocResponse {
            message: "Document dropped successfully".to_string(),
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for sharing a document
pub async fn share_doc_handler(
    State(state): State<AppState>,
    Json(payload): Json<ShareDocRequest>,
) -> Result<Json<ShareDocResponse>, (StatusCode, String)> {
    // Match share mode
    let mode = match payload.mode.to_lowercase().as_str() {
        "read" => ShareMode::Read,
        "write" => ShareMode::Write,
        _ => return Err((StatusCode::BAD_REQUEST, format!("Invalid share mode: {}", payload.mode))),
    };

    // Match address options
    let addr_options = match payload.addr_options.to_lowercase().as_str() {
        "id" => AddrInfoOptions::Id,
        "relayandaddresses" => AddrInfoOptions::RelayAndAddresses,
        "relay" => AddrInfoOptions::Relay,
        "addresses" => AddrInfoOptions::Addresses,
        _ => return Err((StatusCode::BAD_REQUEST, format!("Invalid addr_options: {}", payload.addr_options))),
    };

    match share_doc(state.docs.clone(), payload.doc_id, mode, addr_options).await {
        Ok(ticket) => Ok(Json(ShareDocResponse { ticket })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for joining a document
pub async fn join_doc_handler(
    State(state): State<AppState>,
    Json(payload): Json<JoinDocRequest>,
) -> Result<Json<JoinDocResponse>, (StatusCode, String)> {
    match join_doc(state.docs.clone(), payload.ticket).await {
        Ok(doc_id) => Ok(Json(JoinDocResponse { doc_id })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for closing a document
pub async fn close_doc_handler(
    State(state): State<AppState>,
    Json(payload): Json<CloseDocRequest>,
) -> Result<Json<CloseDocResponse>, (StatusCode, String)> {
    match close_doc(state.docs.clone(), payload.doc_id).await {
        Ok(_) => Ok(Json(CloseDocResponse {
            message: "Document closed successfully".to_string(),
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for adding a document schema

// If the JSON schem looks like this:
/// Example schema:
// ```json
// let schema = r#"{
//     "type": "object",
//     "properties": {
//       "owner": { "type": "string" }
//     },
//     "required": ["owner"]
// }"#;
// ```

// then it should be passed in the request body like this:
// { \"type\": \"object\", \"properties\": { \"owner\": { \"type\": \"string\" } }, \"required\": [\"owner\"] }
pub async fn add_doc_schema_handler(
    State(state): State<AppState>,
    Json(payload): Json<AddDocSchemaRequest>,
) -> Result<Json<AddDocSchemaResponse>, (StatusCode, String)> {
    match add_doc_schema(
        state.docs.clone(),
        payload.author_id,
        payload.doc_id,
        payload.schema,
    ).await {
        Ok(updated_hash) => Ok(Json(AddDocSchemaResponse { updated_hash })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for setting an entry in a document

// Continuing from the previous code snippet, this function sets an entry in a document like this:
// "value": "{\"owner\": \"Dhiway\"}"
pub async fn set_entry_handler(
    State(state): State<AppState>,
    Json(payload): Json<SetEntryRequest>,
) -> Result<Json<SetEntryResponse>, (StatusCode, String)> {
    match set_entry(
        state.docs.clone(),
        state.blobs.clone(),
        payload.doc_id,
        payload.author_id,
        payload.key,
        payload.value,
    )
    .await
    {
        Ok(hash) => Ok(Json(SetEntryResponse { hash })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for setting an entry in a document from a file
pub async fn set_entry_file_handler(
    State(state): State<AppState>,
    Json(payload): Json<SetEntryFileRequest>,
) -> Result<Json<SetEntryFileResponse>, (StatusCode, String)> {
    match set_entry_file(
        state.docs.clone(),
        payload.doc_id,
        payload.author_id,
        payload.key,
        payload.file_path,
    )
    .await
    {
        Ok(outcome) => Ok(Json(SetEntryFileResponse {
            key: outcome.key,
            hash: outcome.hash,
            size: outcome.size,
        })),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

// Handler for getting an entry from a document
pub async fn get_entry_handler(
    State(state): State<AppState>,
    Json(payload): Json<GetEntryRequest>,
) -> Result<Json<GetEntryResponse>, (StatusCode, String)> {
    match get_entry(
        state.docs.clone(),
        payload.doc_id,
        payload.author_id,
        payload.key,
        payload.include_empty,
    ).await {
        Ok(Some(details)) => {
            Ok(Json(GetEntryResponse {
                doc: details.namespace.doc,
                key: details.namespace.key,
                author: details.namespace.author,
                hash: details.record.hash,
                len: details.record.len,
                timestamp: details.record.timestamp,
            }))
        },
        Ok(None) => Err((StatusCode::NOT_FOUND, "Entry not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for getting multiple entries from a document
pub async fn get_entries_handler(
    State(state): State<AppState>,
    Json(payload): Json<GetEntriesRequest>,
) -> Result<Json<Vec<GetEntryResponse>>, (StatusCode, String)> {
    // Parse query_params string into JSON
    let query_params: serde_json::Value = serde_json::from_str(&payload.query_params)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid query_params: {}", e)))?;

    // Fetch entries
    match get_entries(state.docs.clone(), payload.doc_id.clone(), query_params).await {
        Ok(entry_details_vec) => {
            // Convert EntryDetails into GetEntryResponse
            let response_vec = entry_details_vec
                .into_iter()
                .map(|entry| GetEntryResponse {
                    doc: entry.namespace.doc,
                    key: entry.namespace.key,
                    author: entry.namespace.author,
                    hash: entry.record.hash,
                    len: entry.record.len,
                    timestamp: entry.record.timestamp,
                })
                .collect();

            Ok(Json(response_vec))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for deleting an entry from a document
pub async fn delete_entry_handler(
    State(state): State<AppState>,
    Json(payload): Json<DeleteEntryRequest>,
) -> Result<Json<DeleteEntryResponse>, (StatusCode, String)> {
    match delete_entry(
        state.docs.clone(),
        payload.doc_id,
        payload.author_id,
        payload.key,
    ).await {
        Ok(deleted_count) => Ok(Json(DeleteEntryResponse { deleted_count })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for leaving a document
pub async fn leave_handler(
    State(state): State<AppState>,
    Json(payload): Json<LeaveRequest>,
) -> Result<Json<LeaveResponse>, (StatusCode, String)> {
    match leave(state.docs.clone(), payload.doc_id.clone()).await {
        Ok(_) => Ok(Json(LeaveResponse {
            message: format!("Successfully left document {}", payload.doc_id),
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for getting the status of a document
pub async fn status_handler(
    State(state): State<AppState>,
    Json(payload): Json<StatusRequest>,
) -> Result<Json<StatusResponse>, (StatusCode, String)> {
    match status(state.docs.clone(), payload.doc_id.clone()).await {
        Ok(open_state) => Ok(Json(StatusResponse {
            sync: open_state.sync,
            subscribers: open_state.subscribers,
            handles: open_state.handles,
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for setting the download policy of a document
pub async fn set_download_policy_handler(
    State(state): State<AppState>,
    Json(payload): Json<SetDownloadPolicyRequest>,
) -> Result<Json<SetDownloadPolicyResponse>, (StatusCode, String)> {
    let download_policy_value: serde_json::Value = match serde_json::from_str(&payload.download_policy) {
        Ok(val) => val,
        Err(e) => return Err((StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e))),
    };

    match set_download_policy(state.docs.clone(), payload.doc_id, download_policy_value).await {
        Ok(_) => Ok(Json(SetDownloadPolicyResponse {
            message: "Download policy set successfully".to_string(),
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler for getting the download policy of a document
pub async fn get_download_policy_handler(
    State(state): State<AppState>,
    Json(payload): Json<GetDownloadPolicyRequest>,
) -> Result<Json<GetDownloadPolicyResponse>, (StatusCode, String)> {
    match get_download_policy(state.docs.clone(), payload.doc_id).await {
        Ok(policy_value) => {
            match serde_json::to_string_pretty(&policy_value) {
                Ok(policy_str) => Ok(Json(GetDownloadPolicyResponse {
                    download_policy: policy_str,
                })),
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to serialize policy: {}", e))),
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
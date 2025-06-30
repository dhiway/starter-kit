use core::blobs::*;
use helpers::{state::AppState, utils::get_author_id_from_headers};
use iroh_blobs::{
    BlobFormat,
    net_protocol::DownloadMode,
    util::SetTagOption,
    rpc::client::blobs::DownloadOptions,
};
use gateway::access_control::check_node_id_and_domain_header;

use iroh::NodeAddr;
use axum::{extract::State, Json, http::HeaderMap};
use bytes::Bytes;
use serde::Deserialize;
use serde::Serialize;
use iroh_blobs::util::Tag;
use std::str::FromStr;
use iroh_base::PublicKey;
use std::path::PathBuf;

// Request bodies
// 1. add_blob_bytes
#[derive(Deserialize)]
pub struct AddBlobBytesRequest {
    pub content: String, 
}

// 2. add_blob_named
#[derive(Deserialize)]
pub struct AddBlobNamedRequest {
    pub content: String,
    pub name: String,
}

// 3. add_blob_from_path
#[derive(Deserialize)]
pub struct AddBlobFromPathRequest {
    pub file_path: String,
}

// 4. list_blobs
#[derive(Deserialize)]
pub struct ListBlobsRequest {
    pub page: usize,
    pub page_size: usize,
}

// 5. get_blob
#[derive(Deserialize)]
pub struct GetBlobRequest {
    pub hash: String,
}

// 6. status_blob
#[derive(Deserialize)]
pub struct StatusBlobRequest {
    pub hash: String,
}

// 7. has_blob
#[derive(Deserialize)]
pub struct HasBlobRequest {
    pub hash: String,
}

// 8. download_blob
#[derive(Deserialize)]
pub struct DownloadRequest {
    pub hash: String,
    pub node_id: String,
}

// 9. download_hash_sequence
// same as DownloadRequest

// 10. download_with_options
/* example request body:
{
  "hash": "hash_as_string",
  "format": "Raw",
  "mode": "Direct",
  "nodes": ["node_id1", "node_id2"],
  "tag": "Auto"
}
*/
#[derive(Deserialize)]
pub struct DownloadWithOptionsRequest {
    pub hash: String,                     
    pub format: String,
    pub mode: String,
    pub nodes: Vec<String>,
    pub tag: String,
}

// 11. list_tags
// no request body

// 12. delete_tag
#[derive(Deserialize)]
pub struct DeleteTagRequest {
    pub tag_name: String,
}

// 13. export_blob_to_file
#[derive(Deserialize)]
pub struct ExportBlobRequest {
    pub hash: String,
    pub destination: String,
}

// Response bodies
// 1. add_blob_bytes
#[derive(Serialize)]
pub struct AddBlobResponse {
    pub hash: String,
    pub format: String,
    pub size: u64,
    pub tag: String,
}

// 2. add_blob_named
// same as AddBlobResponse

// 3. add_blob_from_path
// same as AddBlobResponse

// 4. list_blobs
#[derive(Serialize)]
pub struct BlobInfoResponse {
    pub path: String,
    pub hash: String,
    pub size: u64,
}

// 5. get_blob
#[derive(Serialize)]
pub struct GetBlobResponse {
    pub content: String,
}

// 6. status_blob
#[derive(Serialize)]
pub struct StatusBlobResponse {
    pub status: String,
}

// 7. has blob
#[derive(Serialize)]
pub struct HasBlobResponse {
    pub present: bool,
}

// 8. download_blob
#[derive(Serialize)]
pub struct DownloadOutcomeResponse {
    pub local_size: u64,
    pub downloaded_size: u64,
    pub stats: String, // Use Debug format for now
}

// 9. download_hash_sequence
// same as DownloadOutcomeResponse

// 10. download_with_options
// same as DownloadOutcomeResponse

// 11. list_tags
#[derive(Serialize)]
pub struct TagInfoResponse {
    pub name: String,
    pub format: String,
    pub hash: String,
}

// 12. delete_tag
#[derive(Serialize)]
pub struct DeleteTagResponse {
    pub message: String,
}

// 13. export_blob_to_file
#[derive(Serialize)]
pub struct ExportBlobResponse {
    pub message: String,
}

// Handler to add blob bytes
pub async fn add_blob_bytes_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AddBlobBytesRequest>,
) -> Result<Json<AddBlobResponse>, (axum::http::StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    let caller_author_id = get_author_id_from_headers(&headers)?;

    // Check if the calling author is in the list of authors
    let authors = core::authors::list_authors(state.docs.clone())
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !authors.contains(&caller_author_id) {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            "Only a registered author can perform this action".to_string(),
        ));
    }

    // request body checks
    if payload.content.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Content cannot be empty".to_string()));
    }

    let bytes = Bytes::from(payload.content);

    match add_blob_bytes(state.blobs.clone(), bytes).await {
        Ok(outcome) => Ok(Json(AddBlobResponse {
            hash: outcome.hash.to_string(),
            format: format!("{:?}", outcome.format),
            size: outcome.size,
            tag: outcome.tag.to_string(),
        })),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to add blob: {}", e),
        )),
    }
}

// Handler to add blob with a name
pub async fn add_blob_named_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AddBlobNamedRequest>,
) -> Result<Json<AddBlobResponse>, (axum::http::StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    let caller_author_id = get_author_id_from_headers(&headers)?;

    // Check if the calling author is in the list of authors
    let authors = core::authors::list_authors(state.docs.clone())
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !authors.contains(&caller_author_id) {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            "Only a registered author can perform this action".to_string(),
        ));
    }

    // request body checks
    if payload.content.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Content cannot be empty".to_string()));
    }
    if payload.name.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    let bytes = Bytes::from(payload.content);
    let tag = Tag::from(payload.name);

    match add_blob_named(state.blobs.clone(), bytes, tag).await {
        Ok(outcome) => Ok(Json(AddBlobResponse {
            hash: outcome.hash.to_string(),
            format: format!("{:?}", outcome.format),
            size: outcome.size,
            tag: outcome.tag.to_string(),
        })),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to add named blob: {}", e),
        )),
    }
}

// Handler to add blob from a file path
pub async fn add_blob_from_path_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AddBlobFromPathRequest>,
) -> Result<Json<AddBlobResponse>, (axum::http::StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    let caller_author_id = get_author_id_from_headers(&headers)?;

    // Check if the calling author is in the list of authors
    let authors = core::authors::list_authors(state.docs.clone())
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !authors.contains(&caller_author_id) {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            "Only a registered author can perform this action".to_string(),
        ));
    }

    // request body checks
    if payload.file_path.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "File path cannot be empty".to_string()));
    }

    let path = std::path::Path::new(&payload.file_path);
    if !path.exists() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "File does not exist".to_string()));
    }

    match add_blob_from_path(state.blobs.clone(), path).await {
        Ok(outcome) => Ok(Json(AddBlobResponse {
            hash: outcome.hash.to_string(),
            format: format!("{:?}", outcome.format),
            size: outcome.size,
            tag: outcome.tag.to_string(),
        })),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to add blob from path: {}", e),
        )),
    }
}

// Handler to list blobs
pub async fn list_blobs_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ListBlobsRequest>,
) -> Result<Json<Vec<BlobInfoResponse>>, (axum::http::StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    // request body checks
    if payload.page_size == 0 {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Page size must be greater than 0".to_string()));
    }

    match list_blobs(state.blobs.clone(), payload.page, payload.page_size).await {
        Ok(blobs) => {
            let response = blobs
                .into_iter()
                .map(|blob| BlobInfoResponse {
                    path: blob.path,
                    hash: blob.hash.to_string(),
                    size: blob.size,
                })
                .collect();
            Ok(Json(response))
        }
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list blobs: {}", e),
        )),
    }
}

// Handler to get a blob by hash
pub async fn get_blob_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<GetBlobRequest>,
) -> Result<Json<GetBlobResponse>, (axum::http::StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    // request body checks
    if payload.hash.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Hash cannot be empty".to_string()));
    }

    match get_blob(state.blobs.clone(), payload.hash).await {
        Ok(content) => Ok(Json(GetBlobResponse { content })),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get blob: {}", e),
        )),
    }
}

// Handler to check the status of a blob
pub async fn status_blob_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<StatusBlobRequest>,
) -> Result<Json<StatusBlobResponse>, (axum::http::StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    // request body checks
    if payload.hash.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Hash cannot be empty".to_string()));
    }

    match status_blob(state.blobs.clone(), payload.hash).await {
        Ok(status) => Ok(Json(StatusBlobResponse { status })),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get blob status: {}", e),
        )),
    }
}

// Handler to check if a blob exists
pub async fn has_blob_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<HasBlobRequest>,
) -> Result<Json<HasBlobResponse>, (axum::http::StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    // request body checks
    if payload.hash.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Hash cannot be empty".to_string()));
    }

    match has_blob(state.blobs.clone(), payload.hash).await {
        Ok(present) => Ok(Json(HasBlobResponse { present })),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to check blob presence: {}", e),
        )),
    }
}

// Handler to download a blob
pub async fn download_blob_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DownloadRequest>,
) -> Result<Json<DownloadOutcomeResponse>, (axum::http::StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    // request body checks
    if payload.hash.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Hash cannot be empty".to_string()));
    }
    if payload.node_id.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Node ID cannot be empty".to_string()));
    }

    match download_blob(state.blobs.clone(), payload.hash, payload.node_id).await {
        Ok(outcome) => Ok(Json(DownloadOutcomeResponse {
            local_size: outcome.local_size,
            downloaded_size: outcome.downloaded_size,
            stats: format!("{:?}", outcome.stats),
        })),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to download blob: {}", e),
        )),
    }
}

// Handler to download a sequence of hashes
// This will not work right now as we have not implemented WarpOption for any function that can create a blob. If 'download_hash_sequence' is required then would need to add that. I think it would be a good feature to have, as then the user could create collections.
pub async fn download_hash_sequence_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DownloadRequest>,
) -> Result<Json<DownloadOutcomeResponse>, (axum::http::StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    // request body checks
    if payload.hash.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Hash cannot be empty".to_string()));
    }
    if payload.node_id.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Node ID cannot be empty".to_string()));
    }

    match download_hash_sequence(state.blobs.clone(), payload.hash, payload.node_id).await {
        Ok(outcome) => Ok(Json(DownloadOutcomeResponse {
            local_size: outcome.local_size,
            downloaded_size: outcome.downloaded_size,
            stats: format!("{:?}", outcome.stats),
        })),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to download hash sequence: {}", e),
        )),
    }
}

// Handler to download a blob with options
pub async fn download_with_options_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<DownloadWithOptionsRequest>,
) -> Result<Json<DownloadOutcomeResponse>, (axum::http::StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    // request body checks
    if req.hash.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Hash cannot be empty".to_string()));
    }
    if req.format.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Format cannot be empty".to_string()));
    }
    if req.mode.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Mode cannot be empty".to_string()));
    }
    if req.nodes.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Nodes cannot be empty".to_string()));
    }
    if req.tag.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Tag cannot be empty".to_string()));
    }

    // Parse format
    let format = match req.format.as_str() {
        "Raw" => BlobFormat::Raw,
        "HashSeq" => BlobFormat::HashSeq,
        _ => return Err((axum::http::StatusCode::BAD_REQUEST, format!("Invalid format: {}", req.format))),
    };

    // Parse mode
    let mode = match req.mode.as_str() {
        "Direct" => DownloadMode::Direct,
        "Queued" => DownloadMode::Queued,
        _ => return Err((axum::http::StatusCode::BAD_REQUEST, format!("Invalid mode: {}", req.mode))),
    };

    // Parse nodes
    let nodes: Vec<NodeAddr> = req.nodes
        .iter()
        .map(|node_id_str| {
            PublicKey::from_str(node_id_str.trim())
                .map(NodeAddr::from)
                .map_err(|e| format!("Invalid node ID '{}': {}", node_id_str, e))
        })
        .collect::<Result<_, _>>()
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e))?;

    // Parse tag
    let tag = if req.tag == "Auto" {
        SetTagOption::Auto
    } else {
        SetTagOption::Named(Tag(Bytes::from(req.tag.clone())))
    };

    // Construct DownloadOptions
    let options = DownloadOptions {
        format,
        nodes,
        tag,
        mode,
    };

    // Call core download function
    match download_with_options(state.blobs.clone(), req.hash, options).await {
        Ok(outcome) => Ok(Json(DownloadOutcomeResponse {
            local_size: outcome.local_size,
            downloaded_size: outcome.downloaded_size,
            stats: format!("{:?}", outcome.stats),
        })),
        Err(e) => Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler to list tags
pub async fn list_tags_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<TagInfoResponse>>, (axum::http::StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    match list_tags(state.blobs.clone()).await {
        Ok(tags) => {
            let response = tags
                .into_iter()
                .map(|tag_info| TagInfoResponse {
                    name: tag_info.name.to_string(),
                    format: match tag_info.format {
                        BlobFormat::Raw => "Raw".to_string(),
                        BlobFormat::HashSeq => "HashSeq".to_string(),
                    },
                    hash: tag_info.hash.to_string(),
                })
                .collect();

            Ok(Json(response))
        }
        Err(e) => Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler to delete a tag
pub async fn delete_tag_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<DeleteTagRequest>,
) -> Result<Json<DeleteTagResponse>, (axum::http::StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    let caller_author_id = get_author_id_from_headers(&headers)?;

    // Check if the calling author is in the list of authors
    let authors = core::authors::list_authors(state.docs.clone())
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !authors.contains(&caller_author_id) {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            "Only a registered author can perform this action".to_string(),
        ));
    }

    // request body checks
    if req.tag_name.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Tag name cannot be empty".to_string()));
    }

    match delete_tag(state.blobs.clone(), req.tag_name.clone()).await {
        Ok(_) => Ok(Json(DeleteTagResponse {
            message: "Tag deleted successfully".to_string(),
        })),
        Err(e) => Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler to export a blob to a file
pub async fn export_blob_to_file_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ExportBlobRequest>,
) -> Result<Json<ExportBlobResponse>, (axum::http::StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    // request body checks
    if req.hash.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Hash cannot be empty".to_string()));
    }
    if req.destination.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Destination cannot be empty".to_string()));
    }

    let path = PathBuf::from(req.destination.clone());

    // what check should we add for the destination path? Can not check if the path exists as it may not exist yet when the request is made. Check on parent directory existance? 
    
    match export_blob_to_file(state.blobs.clone(), req.hash.clone(), path).await {
        Ok(_) => Ok(Json(ExportBlobResponse {
            message: format!("Blob {} exported to {}", req.hash, req.destination),
        })),
        Err(e) => Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
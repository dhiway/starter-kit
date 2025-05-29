use axum::{Router, routing::{get, post}};
use tower_http::cors::CorsLayer;

use crate::api_handlers::{
    authors_handler::*,
    blobs_handler::*,
    docs_handler::*,
};
use crate::helpers::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/blobs/add-blob-bytes", post(add_blob_bytes_handler))
        .route("/blobs/add-blob-named", post(add_blob_named_handler))
        .route("/blobs/add-blob-from-path", post(add_blob_from_path_handler))
        .route("/blobs/list-blobs", get(list_blobs_handler))
        .route("/blobs/get-blob", get(get_blob_handler))
        .route("/blobs/status-blob", get(status_blob_handler))
        .route("/blobs/has-blob", get(has_blob_handler))
        .route("/blobs/download-blob", get(download_blob_handler))
        .route("/blobs/download-hash-sequence", get(download_hash_sequence_handler))
        .route("/blobs/download-with-options", get(download_with_options_handler))
        .route("/blobs/list-tags", get(list_tags_handler))
        .route("/blobs/delete-tag", post(delete_tag_handler))
        .route("/blobs/export-blob-to-file", post(export_blob_to_file_handler))
        .route("/authors/list-authors", get(list_authors_handler))
        .route("/authors/get-default-author", get(get_default_author_handler))
        .route("/authors/set-default-author", post(set_default_author_handler))
        .route("/authors/create-author", post(create_author_handler))
        .route("/authors/delete-author", post(delete_author_handler))
        .route("/authors/verify-author", post(verify_author_handler))
        .route("/docs/get-document", post(get_document_handler))
        .route("/docs/get-entry-blob", post(get_entry_blob_handler))
        .route("/docs/create-document", post(create_doc_handler))
        .route("/docs/list-docs", get(list_docs_handler))
        .route("/docs/drop-doc", post(drop_doc_handler))
        .route("/docs/share-doc", post(share_doc_handler))
        .route("/docs/join-doc", post(join_doc_handler))
        .route("/docs/close-doc", post(close_doc_handler))
        .route("/docs/add-doc-schema", post(add_doc_schema_handler))
        .route("/docs/set-entry", post(set_entry_handler))
        .route("/docs/set-entry-file", post(set_entry_file_handler))
        .route("/docs/get-entry", post(get_entry_handler))
        .route("/docs/get-entries", post(get_entries_handler))
        .route("/docs/delete-entry", post(delete_entry_handler))
        .route("/docs/leave", post(leave_handler))
        .route("/docs/status", get(status_handler))
        .route("/docs/set-download-policy", post(set_download_policy_handler))
        .route("/docs/get-download-policy", get(get_download_policy_handler))
        .with_state(state)
        .layer(CorsLayer::very_permissive())
}
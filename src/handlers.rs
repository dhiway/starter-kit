use std::result::Result::Ok;
use axum::{
    Form,
    Json,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use crate::state::AppState;
use crate::helper::{create_registry, show_all_registry, archive_registry, add_entry, display_entry, delete_entry};
use std::fs;
use std::path::PathBuf;
use serde_json::json;
use serde::Deserialize;
use iroh_docs::NamespaceId;
use std::str::FromStr;
use std::collections::BTreeMap;
use serde_json::Value;

// This module contains the handlers for the API endpoints.
// Registry handlers
pub async fn create_registry_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut registry_name = None;
    let mut schema = None;
    let mut file_path = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "registry_name" => {
                registry_name = Some(field.text().await.unwrap());
            }
            "schema" => {
                schema = Some(field.text().await.unwrap());
            }
            "file" => {
                let file_name = field.file_name().unwrap_or("upload").to_string();
                let data = field.bytes().await.unwrap();

                // Create uploads folder if not exists
                let _ = fs::create_dir_all("./uploads");

                // Write to a permanent location
                let file_path_buf = PathBuf::from(format!("./uploads/{file_name}"));
                fs::write(&file_path_buf, &data).unwrap();

                file_path = Some(file_path_buf.to_string_lossy().to_string());
            }
            _ => {}
        }
    }

    let (registry_name, schema, file_path) = match (registry_name, schema, file_path) {
        (Some(r), Some(s), Some(f)) => (r, s, f),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                "Missing required fields".to_string(),
            )
                .into_response();
        }
    };

    match create_registry(
        state.blobs.clone(),
        state.docs.clone(),
        &registry_name,
        &schema,
        &file_path,
    )
    .await
    {
        Ok(doc_id) => (StatusCode::OK, doc_id.to_string()).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

pub async fn get_all_registries_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let registries = show_all_registry(state.docs.clone(),state.blobs.clone()).await;
    Json(json!({ "registries": registries }))
}

#[derive(Deserialize)]
pub struct ArchiveRegistryForm {
    registry_name: String,
}

pub async fn archive_registry_handler(
    State(state): State<AppState>,
    Form(payload): Form<ArchiveRegistryForm>,
) -> impl IntoResponse {
    match archive_registry(
        state.docs.clone(),
        state.blobs.clone(),
        &payload.registry_name,
    ).await {
        Ok(doc_id) => (StatusCode::OK, doc_id.to_string()).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

// Entry handlers
#[derive(Deserialize)]
pub struct AddEntryForm {
    registry_id: String, // We will receive registry_id from UI
    entry_data: String, // The entry JSON
}

pub async fn add_entry_handler(
    State(state): State<AppState>,
    Form(payload): Form<AddEntryForm>,
) -> impl IntoResponse {
    // println!("Received payload: {:?}", &payload.registry_id);
    let registry_id = match NamespaceId::from_str(&payload.registry_id) {
        Ok(id) => id,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid registry_id: {}", e)).into_response(),
    };

    // Parse the entry_data string into BTreeMap<String, Value>
    let entry_data: BTreeMap<String, Value> = match serde_json::from_str(&payload.entry_data) {
        Ok(data) => data,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid entry_data JSON: {}", e)).into_response(),
    };

    match add_entry(state.docs.clone(), registry_id, entry_data).await {
        Ok(_) => (StatusCode::OK, "Entry added successfully").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
pub struct DisplayEntryForm {
    registry_id: String, // Coming from UI (doc_id)
}

pub async fn display_entry_handler(
    State(state): State<AppState>,
    Form(payload): Form<DisplayEntryForm>,
) -> impl IntoResponse {
    let registry_id = match NamespaceId::from_str(&payload.registry_id) {
        Ok(id) => id,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid registry_id: {}", e)).into_response(),
    };

    match display_entry(state.docs.clone(), state.blobs.clone(), registry_id).await {
        Ok(entries) => Json(json!({ "entries": entries })).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
pub struct DeleteEntryForm {
    registry_id: String,
    entry_id: String,
}

pub async fn delete_entry_handler(
    State(state): State<AppState>,
    Form(payload): Form<DeleteEntryForm>,
) -> impl IntoResponse {
    let registry_id = match NamespaceId::from_str(&payload.registry_id) {
        Ok(id) => id,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid registry_id: {}", e)).into_response(),
    };

    let entry_id = match NamespaceId::from_str(&payload.entry_id) {
        Ok(id) => id,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid entry_id: {}", e)).into_response(),
    };

    match delete_entry(state.docs.clone(), registry_id, entry_id).await {
        Ok(_) => (StatusCode::OK, "Entry deleted successfully").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}
use std::result::Result::Ok;
use axum::{
    Form,
    Json,
    Extension,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::{sync::Arc, fs::File, io::Write};
use tempfile::NamedTempFile;
use crate::state::AppState;
use crate::helper::{create_registry, show_all_registry, archive_registry};
use std::fs;
use std::path::PathBuf;
use serde_json::json;
use serde::Deserialize;

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
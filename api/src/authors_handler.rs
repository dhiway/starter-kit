use helpers::state::AppState;
use gateway::access_control::check_node_id_and_domain_header;

use core::authors::*;
use axum::{extract::State, Json, http::{HeaderMap, StatusCode}};
use serde::{Deserialize, Serialize};

// Request bodies
// 1. list author
// no request body needed

// 2. get default author
// no request body needed

// 3. set default author
#[derive(Deserialize)]
pub struct SetDefaultAuthorRequest {
    pub author_id: String,
}

// 4. create author
// no request body needed

// 5. delete author
#[derive(Deserialize)]
pub struct DeleteAuthorRequest {
    pub author_id: String,
}

// 6. verify author
#[derive(Deserialize)]
pub struct VerifyAuthorRequest {
    pub author_id: String,
}

// Response bodies
// 1. List authors
#[derive(Serialize)]
pub struct AuthorsListResponse {
    pub authors: Vec<String>,
}

// 2. Get default author
#[derive(Serialize)]
pub struct DefaultAuthorResponse {
    pub default_author: String,
}

// 3. Set default author
#[derive(Serialize)]
pub struct SetDefaultAuthorResponse {
    pub message: String,
}

// 4. Create author
#[derive(Serialize)]
pub struct CreateAuthorResponse {
    pub author_id: String,
}

// 5. Delete author
#[derive(Serialize)]
pub struct DeleteAuthorResponse {
    pub message: String,
}

// 6. Verify author
#[derive(Serialize)]
pub struct VerifyAuthorResponse {
    pub is_valid: bool,
}


// handler for listing authors
pub async fn list_authors_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AuthorsListResponse>, (StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    match list_authors(state.docs.clone()).await {
        Ok(authors) => Ok(Json(AuthorsListResponse { authors })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// handler for getting the default author
pub async fn get_default_author_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DefaultAuthorResponse>, (StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    match get_default_author(state.docs.clone()).await {
        Ok(author) => Ok(Json(DefaultAuthorResponse { default_author: author })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// handler for setting the default author
pub async fn set_default_author_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SetDefaultAuthorRequest>,
) -> Result<Json<SetDefaultAuthorResponse>, (StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    // request body checks
    if payload.author_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "author_id cannot be empty".to_string()));
    }

    match set_default_author(state.docs.clone(), payload.author_id).await {
        Ok(_) => Ok(Json(SetDefaultAuthorResponse {
            message: "Default author set successfully".to_string(),
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// handler for creating an author
pub async fn create_author_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CreateAuthorResponse>, (StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    match create_author(state.docs.clone()).await {
        Ok(author_id) => Ok(Json(CreateAuthorResponse { author_id })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// handler for deleting an author
pub async fn delete_author_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DeleteAuthorRequest>,
) -> Result<Json<DeleteAuthorResponse>, (StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    // request body checks
    if payload.author_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "author_id cannot be empty".to_string()));
    }

    match delete_author(state.docs.clone(), payload.author_id).await {
        Ok(()) => Ok(Json(DeleteAuthorResponse { 
            message: "Author deleted successfully".to_string()
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// handler for verifying an author
pub async fn verify_author_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<VerifyAuthorRequest>,
) -> Result<Json<VerifyAuthorResponse>, (StatusCode, String)> {
    check_node_id_and_domain_header(&headers)?;

    // request body checks
    if payload.author_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "author_id cannot be empty".to_string()));
    }

    match verify_author(state.docs.clone(), payload.author_id).await {
        Ok(is_valid) => Ok(Json(VerifyAuthorResponse { is_valid })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
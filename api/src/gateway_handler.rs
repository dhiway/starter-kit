use gateway::access_control::{
    is_node_id_allowed, 
    is_domain_allowed,
    add_node_id,
    remove_node_id,
    add_domain,
    remove_domain
};
use helpers::{
    state::AppState,
    utils::normalize_domain,
};

use serde::{Deserialize, Serialize};
use axum::{extract::State, Json, debug_handler, http::StatusCode};
use anyhow::Result;
use iroh::NodeId;
use std::str::FromStr;
use regex::Regex;

// Request bodies
// 1. is_node_id_allowed
#[derive(Deserialize)]
pub struct IsNodeIdAllowedRequest {
    pub node_id: String,
}

// 2. is_domain_allowed
#[derive(Deserialize)]
pub struct IsDomainAllowedRequest {
    pub domain: String,
}

// 3. add_node_id
#[derive(Deserialize)]
pub struct AddNodeIdRequest {
    pub node_id: String,
}

// 4. remove_node_id
#[derive(Deserialize)]
pub struct RemoveNodeIdRequest {
    pub node_id: String,
}

// 5. add_domain
#[derive(Deserialize)]
pub struct AddDomainRequest {
    pub domain: String,
}

// 6. remove_domain
#[derive(Deserialize)]
pub struct RemoveDomainRequest {
    pub domain: String,
}

// Response bodies
// 1. is_node_id_allowed
#[derive(Serialize)]
pub struct IsNodeIdAllowedResponse {
    pub allowed: bool,
}

// 2. is_domain_allowed
#[derive(Serialize)]
pub struct IsDomainAllowedResponse {
    pub allowed: bool,
}  

// 3. add_node_id
#[derive(Serialize)]
pub struct AddNodeIdResponse {
    pub message: String,
}

// 4. remove_node_id
#[derive(Serialize)]
pub struct RemoveNodeIdResponse {
    pub message: String,
}

// 5. add_domain
#[derive(Serialize)]
pub struct AddDomainResponse {
    pub message: String,
}

// 6. remove_domain
#[derive(Serialize)]
pub struct RemoveDomainResponse {
    pub message: String,
}

// Handler for checking if a node ID is allowed
pub async fn is_node_id_allowed_handler(
    Json(req): Json<IsNodeIdAllowedRequest>
) -> Result<Json<IsNodeIdAllowedResponse>, (StatusCode, String)> {
    if req.node_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "nodeId cannot be empty".to_string()));
    }
    if NodeId::from_str(&req.node_id).is_err() {
        return Err((StatusCode::BAD_REQUEST, "nodeId is not a valid NodeId".to_string()));
    }


    let allowed = is_node_id_allowed(&req.node_id);
    Ok(Json(IsNodeIdAllowedResponse { allowed }))
}

// Handler for checking if a domain is allowed
pub async fn is_domain_allowed_handler(
    Json(req): Json<IsDomainAllowedRequest>
) -> Result<Json<IsDomainAllowedResponse>, (StatusCode, String)> {
    if req.domain.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "domain cannot be empty".to_string()));
    }
    let domain_regex = Regex::new(r"^(https?://)?([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,}$").unwrap();
    if !domain_regex.is_match(&req.domain) {
        return Err((StatusCode::BAD_REQUEST, "Invalid domain format".to_string()));
    }
    
    let normalized = normalize_domain(&req.domain)
        .ok_or((StatusCode::BAD_REQUEST, "Invalid domain format".to_string()))?;


    let allowed = is_domain_allowed(&normalized);
    Ok(Json(IsDomainAllowedResponse { allowed }))
}

// Handler for adding a node ID
#[debug_handler]
pub async fn add_node_id_handler(
    Json(req): Json<AddNodeIdRequest>
) -> Result<Json<AddNodeIdResponse>, (StatusCode, String)> {
    if req.node_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "nodeId cannot be empty".to_string()));
    }
    if NodeId::from_str(&req.node_id).is_err() {
        return Err((StatusCode::BAD_REQUEST, "nodeId is not a valid NodeId".to_string()));
    }

    add_node_id(req.node_id.clone()).await;
    Ok(Json(AddNodeIdResponse { message: "Node ID added successfully".to_string() }))
}

// Handler for removing a node ID
pub async fn remove_node_id_handler(
    Json(req): Json<RemoveNodeIdRequest>
) -> Result<Json<RemoveNodeIdResponse>, (StatusCode, String)> {
    if req.node_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "nodeId cannot be empty".to_string()));
    }
    if NodeId::from_str(&req.node_id).is_err() {
        return Err((StatusCode::BAD_REQUEST, "nodeId is not a valid NodeId".to_string()));
    }

    remove_node_id(&req.node_id).await;
    Ok(Json(RemoveNodeIdResponse { message: "Node ID removed successfully".to_string() }))
}

// Handler for adding a domain
pub async fn add_domain_handler(
    Json(req): Json<AddDomainRequest>
) -> Result<Json<AddDomainResponse>, (StatusCode, String)> {
    if req.domain.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "domain cannot be empty".to_string()));
    }
    // TODO: Add domain validation if necessary
    let domain_regex = Regex::new(r"^(https?://)?([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,}$").unwrap();
    if !domain_regex.is_match(&req.domain) {
        return Err((StatusCode::BAD_REQUEST, "Invalid domain format".to_string()));
    }
    
    let normalized = normalize_domain(&req.domain)
        .ok_or((StatusCode::BAD_REQUEST, "Invalid domain format".to_string()))?;

    add_domain(normalized.clone()).await;
    Ok(Json(AddDomainResponse { message: "Domain added successfully".to_string() }))
}

// Handler for removing a domain
pub async fn remove_domain_handler(
    Json(req): Json<RemoveDomainRequest>
) -> Result<Json<RemoveDomainResponse>, (StatusCode, String)> {
    if req.domain.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "domain cannot be empty".to_string()));
    }
    // TODO: Add domain validation if necessary
    let domain_regex = Regex::new(r"^(https?://)?([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,}$").unwrap();
    if !domain_regex.is_match(&req.domain) {
        return Err((StatusCode::BAD_REQUEST, "Invalid domain format".to_string()));
    }
    
    let normalized = normalize_domain(&req.domain)
        .ok_or((StatusCode::BAD_REQUEST, "Invalid domain format".to_string()))?;

    remove_domain(&normalized).await;
    Ok(Json(RemoveDomainResponse { message: "Domain removed successfully".to_string() }))
}
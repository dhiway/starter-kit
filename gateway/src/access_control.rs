use crate::storage::{save_set};
use helpers::utils::normalize_domain;

use std::collections::HashSet;
use std::sync::RwLock;
use lazy_static::lazy_static;
use axum::http::{HeaderMap, StatusCode};

lazy_static! {
    static ref NODE_IDS: RwLock<HashSet<String>> = RwLock::new(HashSet::new());
    static ref DOMAINS: RwLock<HashSet<String>> = RwLock::new(HashSet::new());
}

static mut STORAGE_PATH: Option<String> = None;

pub fn set_storage_path(path: String, node_ids: HashSet<String>, domains: HashSet<String>) {
    unsafe {
        STORAGE_PATH = Some(path);
    }
    *NODE_IDS.write().unwrap() = node_ids;
    *DOMAINS.write().unwrap() = domains;
}

pub fn is_node_id_allowed(node_id: &str) -> bool {
    NODE_IDS.read().unwrap().contains(node_id)
}

pub fn is_domain_allowed(domain: &str) -> bool {
    DOMAINS.read().unwrap().contains(domain)
}

pub async fn add_node_id(node_id: String) {
    {
        let mut ids = NODE_IDS.write().unwrap();
        ids.insert(node_id.clone());
        // lock is dropped here
    }
    let ids_snapshot = {
        let ids = NODE_IDS.read().unwrap();
        ids.clone()
    };
    save("allowed_node_ids.json", &ids_snapshot).await;
}

pub async fn remove_node_id(node_id: &str) {
    {
        let mut ids = NODE_IDS.write().unwrap();
        ids.remove(node_id);
        // lock is dropped here
    }
    let ids_snapshot = {
        let ids = NODE_IDS.read().unwrap();
        ids.clone()
    };
    save("allowed_node_ids.json", &ids_snapshot).await;
}

pub async fn add_domain(domain: String) {
    {
        let mut domains = DOMAINS.write().unwrap();
        domains.insert(domain.clone());
        // lock is dropped here
    }
    let domains_snapshot = {
        let domains = DOMAINS.read().unwrap();
        domains.clone()
    };
    save("allowed_domains.json", &domains_snapshot).await;
}

pub async fn remove_domain(domain: &str) {
    {
        let mut domains = DOMAINS.write().unwrap();
        domains.remove(domain);
        // lock is dropped here
    }
    let domains_snapshot = {
        let domains = DOMAINS.read().unwrap();
        domains.clone()
    };
    save("allowed_domains.json", &domains_snapshot).await;
}

async fn save(filename: &str, set: &HashSet<String>) {
    if let Some(path) = unsafe { STORAGE_PATH.clone() } {
        let _ = save_set(&path, filename, set).await;
    }
}

pub async fn ensure_self_node_id_allowed(path: &str, node_id: String, node_ids: &mut HashSet<String>) -> anyhow::Result<()> {
    if node_ids.is_empty() {
        println!(
            "🟢 First run: Added this node's own NodeId ({}) to the allowed list.\n\
             ℹ️  To allow other nodes to interact with your data, add their NodeIds using the appropriate API.\n",
            node_id
        );
        node_ids.insert(node_id.clone());
        save_set(path, "allowed_node_ids.json", node_ids).await?;
    }
    Ok(())
}

// Check if the request has a valid nodeId header
// TODO: add check for domain too
pub fn check_node_id_and_domain_header(headers: &HeaderMap) -> Result<(), (StatusCode, String)> {
    let node_id = headers.get("nodeId").and_then(|v| v.to_str().ok());
    let origin = headers.get("Origin").and_then(|v| v.to_str().ok());

    match (node_id, origin) {
        (Some(nid), None) => {
            if !is_node_id_allowed(nid) {
                return Err((
                    StatusCode::FORBIDDEN,
                    "Access denied for this nodeId".to_string(),
                ));
            }
        }
        (None, Some(origin_str)) => {
            let domain = normalize_domain(origin_str)
                .ok_or((StatusCode::BAD_REQUEST, "Invalid Origin header format".to_string()))?;

            if !is_domain_allowed(&domain) {
                return Err((
                    StatusCode::FORBIDDEN,
                    format!("Access denied for domain: {}", domain),
                ));
            }
        }
        (Some(_), Some(_)) => {
            // TODO: Handle case where both nodeId and Origin are provided
        }
        (None, None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Missing both nodeId and Origin headers".to_string(),
            ));
        }
    }

    Ok(())
}
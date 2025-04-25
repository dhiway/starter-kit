use anyhow::{anyhow, Result};
use iroh_blobs::Hash;
use crate::blobs::save_file_to_blobs;
use crate::docs::{save_as_doc, set_value, fetch_doc_as_json};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use iroh_docs::NamespaceId;
use std::sync::{Arc, Mutex};
use iroh_blobs::net_protocol::Blobs;
use iroh_docs::protocol::Docs;
use iroh_blobs::store::mem::Store as BlobStore;
use std::path::Path;
use lazy_static::lazy_static;

// === Global Mappings ===
lazy_static! {
    static ref REGISTRY_SCHEMA_MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref REGISTRY_DOCID_MAP: Mutex<HashMap<String, NamespaceId>> = Mutex::new(HashMap::new());
}

pub async fn create_registry(
    blobs: Arc<Blobs<BlobStore>>, 
    docs: Arc<Docs<BlobStore>>, 
    registry_name: &str,
    schema_str: &str, 
    file_path: &str
) -> Result<NamespaceId, Box<dyn std::error::Error>> {
    // Step 1: Parse schema as JSON
    let parsed_schema: Value = serde_json::from_str(schema_str)
        .map_err(|e| anyhow!("Invalid JSON schema: {}", e))?;

    if !parsed_schema.is_object() {
        return Err(anyhow!("Schema must be a JSON object").into());
    }

    // Step 2: Save file to blobs and get hash
    let hash = save_file_to_blobs(blobs.clone(), Path::new(file_path), true).await?;

    // Step 3: Compose document JSON
    let mut doc_data = BTreeMap::new();
    doc_data.insert("registry_name".to_string(), json!(registry_name));
    doc_data.insert("schema".to_string(), parsed_schema.clone()); // Keep as JSON object
    doc_data.insert("file".to_string(), json!(hash));
    doc_data.insert("archived".to_string(), json!(false));

    // Step 4: Save as document
    let doc_id = save_as_doc(docs.clone(), doc_data).await?;

    // Step 5: Update global maps
    {
        let mut schema_map = REGISTRY_SCHEMA_MAP.lock().unwrap();
        schema_map.insert(registry_name.to_string(), schema_str.to_string());
    }
    {
        let mut docid_map = REGISTRY_DOCID_MAP.lock().unwrap();
        docid_map.insert(registry_name.to_string(), doc_id.clone());
    }

    Ok(doc_id)
}

pub fn show_all_registry() {
    println!("=== All Registries ===");

    let schema_map = REGISTRY_SCHEMA_MAP.lock().unwrap();
    let docid_map = REGISTRY_DOCID_MAP.lock().unwrap();

    for (registry_name, schema_str) in schema_map.iter() {
        let doc_id = docid_map.get(registry_name);
        println!("Registry Name: {}", registry_name);
        println!("Schema: {}", schema_str);
        match doc_id {
            Some(id) => println!("Doc ID: {}\n", id),
            None => println!("Doc ID: Not found\n"),
        }
    }
}

pub async fn archive_registry(
    docs: Arc<Docs<BlobStore>>, 
    blobs: Arc<Blobs<BlobStore>>,
    registry_name: &str
) -> Result<Hash, Box<dyn std::error::Error>> {
    let registry_map = REGISTRY_DOCID_MAP.lock().unwrap();
    let Some(doc_id) = registry_map.get(registry_name) else {
        return Err(format!("No document found for registry: {}", registry_name).into());
    };

    let doc_client = docs.client();
    let Some(doc) = doc_client.open(doc_id.clone()).await? else {
        return Err(format!("Document not found: {}", doc_id).into());
    };

    let doc_json = fetch_doc_as_json(docs.clone(), blobs.clone() , *doc_id).await?;
    if let Some(Value::Bool(true)) = doc_json.get("archived") {
        return Err("Registry is already archived.".into());
    }

    let result_hash = set_value(
        docs.clone(),
        *doc_id,
        "archived".to_string(),
        Value::Bool(true),
    ).await?;

    Ok(result_hash)
}
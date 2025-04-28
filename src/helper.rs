use anyhow::{anyhow, Result};
use iroh_blobs::Hash;
use crate::blobs::save_file_to_blobs;
use crate::docs::{save_as_doc, set_value, fetch_doc_as_json, delete_doc};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap, HashSet};
use iroh_docs::NamespaceId;
use std::sync::Arc;
use tokio::sync::Mutex;
use iroh_blobs::net_protocol::Blobs;
use iroh_docs::protocol::Docs;
use iroh_blobs::store::mem::Store as BlobStore;
use std::path::Path;
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};

// === Global Mappings ===
lazy_static! {
    static ref REGISTRY_SCHEMA_MAP: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref REGISTRY_DOCID_MAP: Arc<Mutex<HashMap<String, NamespaceId>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref REGISTRY_ENTRIES_MAP: Arc<Mutex<HashMap<NamespaceId, Vec<NamespaceId>>>> = Arc::new(Mutex::new(HashMap::new()));
}

// Registry functions

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
        let mut schema_map = REGISTRY_SCHEMA_MAP.lock().await;
        schema_map.insert(registry_name.to_string(), schema_str.to_string());
        println!("mapping: {:?}", schema_map);
    }
    {
        let mut docid_map = REGISTRY_DOCID_MAP.lock().await;
        docid_map.insert(registry_name.to_string(), doc_id.clone());
        println!("mapping: {:?}", docid_map);
    }
    {
        let mut entry_map = REGISTRY_ENTRIES_MAP.lock().await;
        entry_map.insert(doc_id, Vec::new()); 
        println!("mapping: {:?}", entry_map);
    }

    Ok(doc_id)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    pub registry_name: String,
    pub schema: String,
    pub file: Option<Value>,
    pub archived: Option<Value>,
    pub doc_id: String,
}

pub async fn show_all_registry(
    docs: Arc<Docs<BlobStore>>,
    blobs: Arc<Blobs<BlobStore>>,
) -> Vec<Registry> {
    let docid_map = REGISTRY_DOCID_MAP.lock().await;

    let mut registries = Vec::new();

    for (name, doc_id) in docid_map.iter() {
        // let doc_id: NamespaceId = match doc_id_str.parse() {
        //     Ok(id) => id,
        //     Err(_) => continue,
        // };

        match fetch_doc_as_json(docs.clone(), blobs.clone(), *doc_id, None).await {
            Ok(json_map) => {
                let registry_name = json_map
                    .get("registry_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(name)
                    .to_string();

                let schema_value = json_map
                    .get("schema")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                
                let schema = schema_value.to_string();

                    registries.push(Registry {
                        registry_name,
                        schema,
                        file: json_map.get("file").cloned(),
                        archived: json_map.get("archived").cloned(),
                        doc_id: hex::encode(doc_id.as_bytes()),
                });
            }
            Err(e) => {
                eprintln!("❌ Error fetching doc {} ({}): {}", name, doc_id, e);
            }
        }
    }

    registries
}

pub async fn archive_registry(
    docs: Arc<Docs<BlobStore>>, 
    blobs: Arc<Blobs<BlobStore>>,
    registry_name: &str
) -> Result<Hash, Box<dyn std::error::Error>> {
    let registry_map = REGISTRY_DOCID_MAP.lock().await;
    let Some(doc_id) = registry_map.get(registry_name) else {
        return Err(format!("No document found for registry: {}", registry_name).into());
    };

    let doc_client = docs.client();
    let Some(doc) = doc_client.open(doc_id.clone()).await? else {
        return Err(format!("Document not found: {}", doc_id).into());
    };

    let doc_json = fetch_doc_as_json(docs.clone(), blobs.clone() , *doc_id, None).await?;
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

// Entry functions 

// add a new entry
pub async fn add_entry(
    docs: Arc<Docs<BlobStore>>,
    registry_id: NamespaceId,
    mut json: BTreeMap<String, Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Step 0: Find registry name from registry_id
    let docid_map = REGISTRY_DOCID_MAP.lock().await;
    let registry_name = docid_map
        .iter()
        .find_map(|(name, id)| if *id == registry_id { Some(name.clone()) } else { None })
        .ok_or("Registry ID not found in docid map")?;

    // Step 1: Fetch the schema for this registry
    let schema_map = REGISTRY_SCHEMA_MAP.lock().await;
    let schema_json_str = schema_map
        .get(&registry_name)
        .ok_or("Registry name not found in schema map")?;

    let schema_json: Value = serde_json::from_str(schema_json_str)?;

    // Step 2: Parse schema keys into a HashSet
    let mut schema_keys = HashSet::new();
    if let Value::Object(map) = schema_json {
        for (key, _type) in map {
            schema_keys.insert(key);
        }
    } else {
        return Err("Invalid schema format".into());
    }

    // Step 3: Get keys from input JSON
    let input_keys: HashSet<String> = json.keys().cloned().collect();

    // Step 4: Compare keys
    if input_keys != schema_keys {
        return Err("Input JSON keys do not match schema keys".into());
    }

    // Step 5: Save the JSON as a new document
    let entry_id = save_as_doc(docs.clone(), json.clone()).await?; // Save the JSON as a new document

    // Step 6: Add entry_id into the JSON itself
    json.insert("entry_id".to_string(), Value::String(entry_id.to_string()));

    let mut entries_map = REGISTRY_ENTRIES_MAP.lock().await;

    entries_map
        .entry(registry_id)
        .or_insert_with(Vec::new)
        .push(entry_id); // Add the new entry ID into the list for this registry

    println!("entries: {:?}", entries_map);

    Ok(())
}

// display all entries
pub async fn display_entry(
    docs: Arc<Docs<BlobStore>>,
    blobs: Arc<Blobs<BlobStore>>,
    registry_id: NamespaceId,
) -> Result<Vec<BTreeMap<String, Value>>, Box<dyn std::error::Error>> {
    let schema_map = REGISTRY_SCHEMA_MAP.lock().await;
    let entry_map = REGISTRY_ENTRIES_MAP.lock().await;

    let docid_map = REGISTRY_DOCID_MAP.lock().await;
    let registry_name = docid_map
        .iter()
        .find_map(|(name, id)| if *id == registry_id { Some(name.clone()) } else { None })
        .ok_or("Registry ID not found in docid map")?;

    // Step 1: Get schema string for the registry_id
    let schema_json_str = schema_map
        .get(&registry_name)
        .ok_or("Registry name not found in schema map")?;

    // Step 2: Parse schema string to get keys
    let schema_json: Value = serde_json::from_str(schema_json_str)?;
    let mut keys_vec = Vec::new();

    if let Value::Object(map) = schema_json {
        for (key, _value) in map {
            keys_vec.push(key);
        }
    } else {
        return Err("Invalid schema format".into());
    }

    // Step 3: Get the list of entry IDs for this registry
    let entry_ids = entry_map
        .get(&registry_id)
        .ok_or("Registry ID not found in entry map")?;

    let mut entries = Vec::new();

    // Step 4: For each entry_id, fetch the document as JSON
    for entry_id in entry_ids {
        let mut entry_json = fetch_doc_as_json(
            docs.clone(),
            blobs.clone(),
            *entry_id,
            Some(keys_vec.iter().map(|s| s.as_str()).collect()),
        ).await?;

        entry_json.insert("id".to_string(), Value::String(entry_id.to_string()));

        entries.push(entry_json);
    }
    println!("entries: {:?}", entries);

    Ok(entries)
}

// delete entry
pub async fn delete_entry(
    docs: Arc<Docs<BlobStore>>,
    registry_id: NamespaceId,
    entry_id: NamespaceId,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries_map = REGISTRY_ENTRIES_MAP.lock().await;
    println!("entries_map: {:?}", entries_map);

    // Check if the registry exists
    if let Some(entry_list) = entries_map.get_mut(&registry_id) {
        // Check if the entry exists
        if let Some(pos) = entry_list.iter().position(|id| *id == entry_id) {
            // First, delete the doc
            delete_doc(docs.clone(), entry_id).await?;

            // Then, remove the entry from the vec
            entry_list.remove(pos);

            println!("Entry deleted successfully");

            Ok(())
        } else {
            Err(format!("Entry ID not found in the registry").into())
        }
    } else {
        Err(format!("Registry ID not found").into())
    }
}
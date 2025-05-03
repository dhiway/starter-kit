use iroh_blobs::net_protocol::Blobs;
use iroh_blobs::{get, Hash};
use iroh_docs::protocol::Docs;
use iroh_blobs::store::mem::Store as BlobStore;
use iroh_docs::rpc::AddrInfoOptions;
use iroh_docs::{AuthorId, CapabilityKind, DocTicket, NamespaceId};
use iroh_docs::rpc::client::docs::{Doc, MemClient, ShareMode};
use iroh_docs::rpc::proto::RpcService as DocsRpcService;
use jsonschema::draft201909::meta::validate;
use jsonschema::validator_for;
// use quic_rpc::client::FlumeConnector;
// iroh_docs::rpc::client::docs::MemClient;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use serde_json::Value;
use bytes::Bytes;
use quic_rpc::transport::Connector;
// use quic_rpc::client::BoxedConnector;

/// Save a BTreeMap<String, Value> as a new document in iroh-docs.
pub async fn save_as_doc(
    docs: Arc<Docs<BlobStore>>,
    json: BTreeMap<String, Value>,
) -> Result<NamespaceId, Box<dyn std::error::Error>> {
    let doc_client = docs.client();
    let author_id = doc_client.authors().default().await?;
    println!("author_id: {}", author_id);

    let doc = doc_client
        .create()
        .await?;

    for (key, value) in json {
        let bytes = serde_json::to_vec(&value)?;
        doc
            .set_bytes(
                author_id.clone(),
                key.into_bytes(),
                bytes,
            )
            .await?;
    }

    println!("Document saved with ID: {}", doc.id());
    Ok(doc.id())
}

pub async fn fetch_doc_as_json(
    docs: Arc<Docs<BlobStore>>,
    blobs: Arc<Blobs<BlobStore>>,
    doc_id: NamespaceId,
    keys: Option<Vec<&str>>,
) -> Result<BTreeMap<String, Value>, Box<dyn std::error::Error>> {
    let doc_client = docs.client();

    let Some(doc) = doc_client.open(doc_id).await? else {
        return Err("Document not found".into());
    };

    let author = doc_client.authors().default().await?;
    println!("authors: {:?}", author);

    let blob_client = blobs.client();
    let mut result_map = BTreeMap::new();
    let keys = keys.unwrap_or_else(|| vec!["registry_name", "schema", "file", "archived"]);

    for key in keys {
        if let Some(entry) = doc.get_exact(author, key, false).await? {
            let hash = entry.content_hash();

            let read_to_bytes = blob_client
                .read_to_bytes(hash)
                .await?;
            let decoded_str = std::str::from_utf8(&read_to_bytes)?;
            let value: Value = serde_json::from_str(decoded_str).unwrap();
            result_map.insert(key.to_string(), value);
        }
    }

    let json_output = serde_json::to_string_pretty(&result_map)?;
    println!("{}", json_output);

    Ok(result_map)
}

pub async fn set_value(
    docs: Arc<Docs<BlobStore>>,
    doc_id: NamespaceId,
    key: String,
    value: Value
) -> Result<Hash, Box<dyn std::error::Error>> {
    let doc_client = docs.client();

    let Some(doc) = doc_client.open(doc_id).await? else {
        return Err("Document not found".into());
    };

    let author = doc_client.authors().default().await?;

    let key_bytes = Bytes::from(key.clone());
    let value_bytes = Bytes::from(serde_json::to_vec(&value)?);

    let updated_hash = doc.set_bytes(
        author,
        key_bytes,
        value_bytes,
    ).await?;

    Ok(updated_hash)
}

pub async fn delete_doc(
    docs: Arc<Docs<BlobStore>>,
    doc_id: NamespaceId
) -> Result<(), Box<dyn std::error::Error>> {
    let doc_client = docs.client();

    let Some(_doc) = doc_client.open(doc_id).await? else {
        return Err("Document not found".into());
    };

    doc_client.drop_doc(doc_id).await?;

    Ok(())
}

/////////////////////////

use quic_rpc::transport::flume::FlumeConnector;
use iroh_docs::rpc::proto::{Request, Response};
use anyhow::{Result, Context};
use futures::TryStreamExt;
use futures::StreamExt;
use iroh_docs::store::{Query, SortBy, SortDirection};
use crate::utils::{encode_doc_id, decode_doc_id, encode_key, decode_key, SS58AuthorId, ApiDownloadPolicy, validate_key};
use std::str::FromStr;
use serde::Serialize;
use iroh_docs::actor::OpenState;

/// Retrieves a document by its ID.
/// 
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
/// * `doc_id` - The unique document namespace ID.
/// 
/// # Returns
/// * `Doc` - The opened document client.
pub async fn get_document(
    docs: Arc<Docs<BlobStore>>,
    doc_id: NamespaceId,
) -> anyhow::Result<Doc<FlumeConnector<Response, Request>>> {
    let doc_client = docs.client(); 

    let doc = doc_client
        .open(doc_id)
        .await
        .with_context(|| format!("Failed to open document {doc_id}"))?
        .ok_or_else(|| anyhow::anyhow!("Document not found: {doc_id}"))?;

    Ok(doc)
}

/// Reads and decodes a blob entry from storage.
/// 
/// # Arguments
/// * `blobs` - The Arc-wrapped Blobs client.
/// * `hash` - The hash of the blob to retrieve.
/// 
/// # Returns
/// * `String` - The UTF-8 decoded blob content.
pub async fn get_blob_entry(
    blobs: Arc<Blobs<BlobStore>>,
    hash: Hash,
) -> anyhow::Result<String> {
    let blob_client = blobs.client();

    let read_to_bytes = blob_client
        .read_to_bytes(hash)
        .await
        .with_context(|| format!("Failed to read blob {hash}"))?;

    let decoded_str = std::str::from_utf8(&read_to_bytes)
        .with_context(|| format!("Failed to decode blob {hash}"))?;

    Ok(decoded_str.to_string())
}

/// Creates a new document and returns its encoded ID.
/// 
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
/// 
/// # Returns
/// * `String` - The base64-encoded document ID.
pub async fn create_doc(
    docs: Arc<Docs<BlobStore>>,
) -> anyhow::Result<String> {
    let doc_client = docs.client();

    let doc = doc_client
        .create()
        .await
        .with_context(|| "Failed to create document")?;

    let doc_id = encode_doc_id(doc.id().as_bytes());

    Ok(doc_id)
}

/// Lists all documents along with their capability types.
/// 
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
/// 
/// # Returns
/// * `Vec<(String, CapabilityKind)>` - A list of encoded document IDs and their capabilities.
pub async fn list_docs(
    docs: Arc<Docs<BlobStore>>,
) -> anyhow::Result<Vec<(String, CapabilityKind)>> {
    let doc_client = docs.client();

    let mut docs_stream = doc_client
        .list()
        .await
        .with_context(|| "Failed to list documents")?;

    let mut doc_list = Vec::new();

    while let Some((namespace_id, capability)) = docs_stream
        .try_next()
        .await
        .with_context(|| "Error while streaming document list")?
    {
        let doc_id = encode_doc_id(namespace_id.as_bytes());
        doc_list.push((doc_id, capability));
    }

    Ok(doc_list)
}

/// Deletes a document by its encoded ID.
/// 
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
/// * `doc_id` - The base64-encoded document ID to delete.
/// 
/// # Returns
/// * `()` - Indicates successful deletion.
pub async fn drop_doc(
    docs: Arc<Docs<BlobStore>>,
    doc_id: String,
) -> anyhow::Result<()> {
    let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
    let namespace_id = NamespaceId::from(namespace_id_vec);

    let doc_client = docs.client();

    doc_client
        .drop_doc(namespace_id)
        .await
        .with_context(|| format!("Failed to drop document {namespace_id}"))?;

    Ok(())
}

/// Shares a document using the given mode and address options.
/// 
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
/// * `doc_id` - The base64-encoded document ID to share.
/// * `mode` - The sharing mode (read/write).
/// * `addr_options` - Peer address options to include.
/// 
/// # Returns
/// * `String` - The generated document share ticket.
pub async fn share_doc(
    docs: Arc<Docs<BlobStore>>,
    doc_id: String,
    mode: ShareMode,
    addr_options: AddrInfoOptions,
) -> anyhow::Result<String> {
    let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
    let namespace_id = NamespaceId::from(namespace_id_vec);

    let doc = get_document(docs, namespace_id).await?;

    let doc_ticket = doc
        .share(mode, addr_options)
        .await
        .with_context(|| format!("Failed to share document {namespace_id}"))?;

    Ok(doc_ticket.to_string())
}

/// Joins a shared document using its ticket.
/// 
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
/// * `ticket` - The share ticket string.
/// 
/// # Returns
/// * `String` - The namespace ID of the joined document.
pub async fn join_doc(
    docs: Arc<Docs<BlobStore>>,
    ticket: String,
) -> anyhow::Result<String> {
    let doc_ticket = DocTicket::from_str(&ticket)
        .with_context(|| format!("Failed to parse document ticket {ticket}"))?;

    let doc_client = docs.client();

    let (doc_id, _) = doc_client
        .import_and_subscribe(doc_ticket)
        .await
        .with_context(|| "Failed to join document")?;

    Ok(doc_id.id().to_string())
}

/// Closes an open document.
/// 
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
/// * `doc_id` - The base64-encoded document ID to close.
/// 
/// # Returns
/// * `()` - Indicates successful closure.
pub async fn close_doc(
    docs: Arc<Docs<BlobStore>>,
    doc_id: String,
) -> anyhow::Result<()> {
    let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
    let namespace_id = NamespaceId::from(namespace_id_vec);

    let doc = get_document(docs, namespace_id).await?;

    doc.close()
        .await
        .with_context(|| format!("Failed to close document {namespace_id}"))?;

    Ok(())
}

/// Adds a JSON Schema to a document if it's currently empty.
/// 
/// This schema acts as a contract for all future entries in the document.
/// The schema must be a valid JSON Schema, and it will be stored under the key `"schema"`.
///
/// Example schema:
/// ```json
/// let schema = r#"{
///     "type": "object",
///     "properties": {
///       "owner": { "type": "string" },
///       "name": { "type": "string" },
///       "number_of_entries": { "type": "integer" },
///       "terms_and_conditions": { "type": "string" }
///     },
///     "required": ["owner", "name", "number_of_entries", "terms_and_conditions"]
/// }"#;
/// ```
pub async fn add_doc_schema(
    docs: Arc<Docs<BlobStore>>,
    author_id: String,
    doc_id: String,
    schema: String,
) -> anyhow::Result<String> {
    let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
    let namespace_id = NamespaceId::from(namespace_id_vec);

    let schema_json_bytes = serde_json::from_str(&schema)
        .context("Failed to serialize schema to JSON")?;

    validator_for(&schema_json_bytes)
        .context("Failed to validate schema")?;

    let author = SS58AuthorId::decode(&author_id)
        .with_context(|| format!("Failed to decode author ID {author_id}"))?;

    let doc = get_document(docs, namespace_id).await?;

    let mut entries_stream = doc.get_many(Query::all()).await?;
    if entries_stream.next().await.is_some() {
        anyhow::bail!("Document already contains entries. Schema can only be added to an empty document.");
    }

    let key = "schema";
    let encoded_key = encode_key(key.as_bytes());

    let updated_hash = doc
        .set_bytes(
            author,
            encoded_key,
            schema.into_bytes(),
        )
        .await
        .context("Failed to set schema bytes in document")?;

    Ok(updated_hash.to_string())
}

/// Adds a new entry (key-value pair) to the document after validating it against the schema, if one exists.
///
/// If a schema is present in the document, the entry must conform to it.
///
/// Example entry(according to the schema used in `add_doc_schema` comments):
/// ```json
/// let entry_1 = json!({
///     "owner": "Dhiway",
///     "name": "Cyra",
///     "number_of_entries": 3,
///     "terms_and_conditions": "Agreed"
/// });
/// ```
pub async fn set_entry(
    docs: Arc<Docs<BlobStore>>,
    blobs: Arc<Blobs<BlobStore>>,
    doc_id: String,
    author_id: String,
    key: String,
    value: String,
) -> anyhow::Result<String> {
    let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
    let namespace_id = NamespaceId::from(namespace_id_vec);

    let author = SS58AuthorId::decode(&author_id)
        .with_context(|| format!("Failed to decode author ID {author_id}"))?;

    // validate key
    validate_key(&key, true)
        .await
        .context("Failed to validate key")?;

    // get doc
    let doc = get_document(docs, namespace_id).await?;

    // check if there is any value corresponding to the key 'schema' 
    let schema_key = "schema";
    let encoded_schema_key = encode_key(schema_key.as_bytes());
    let blob_client = blobs.client();

    if let Some(schema_entry) = doc
        .get_exact(author, encoded_schema_key.clone(), true)
        .await
        .context("Failed to get schema entry")?
    {
        // get the hash of that entry
        let schema_entry_hash = schema_entry.content_hash();

        // get the data for that blob
        let schema_to_bytes = blob_client
            .read_to_bytes(schema_entry_hash)
            .await
            .context("Failed to read schema blob")?;

        // convert the blob data to JSON
        let schema_str = std::str::from_utf8(&schema_to_bytes)?;
        let schema_json: Value = serde_json::from_str(schema_str).unwrap();

        let validator = validator_for(&schema_json)
            .context("Failed to create JSON schema validator")?;

        // convert value to JSON
        let value_json: Value = serde_json::from_str(&value)
            .context("Failed to convert value to JSON")?;

        // validate the value against the schema
        if !validator.is_valid(&value_json) {
            return Err(anyhow::anyhow!("Value does not match schema"));
        }
    }

    // put the key-value pair in the document
    let encoded_key = encode_key(key.as_bytes());
    let hash = doc
        .set_bytes(author, encoded_key, value.into_bytes())
        .await
        .context("Failed to set entry bytes in document")?;

    Ok(hash.to_string())
}

/// Adds a file as an entry to the document, only if no schema is defined.
///
/// # Parameters
/// - `docs`: Shared reference to the document store.
/// - `doc_id`: Document ID to which the file will be added.
/// - `author_id`: SS58-encoded author ID.
/// - `key`: Key under which the file will be stored in the document.
/// - `file_path`: Path to the file to import.
///
/// # Returns
/// - Outcome including key, hash, and size of the imported file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImportFileOutcome {
    /// The key of the entry
    pub key: String,
    /// The hash of the entry's content
    pub hash: String,
    /// The size of the entry
    pub size: u64,
}

pub async fn set_entry_file (
    docs: Arc<Docs<BlobStore>>,
    doc_id: String,
    author_id: String,
    key: String,
    file_path: String,
) -> anyhow::Result<ImportFileOutcome> {
    let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
    let namespace_id = NamespaceId::from(namespace_id_vec);

    let author = SS58AuthorId::decode(&author_id)
        .with_context(|| format!("Failed to decode author ID {author_id}"))?;

    validate_key(&key, true)
        .await
        .context("Failed to validate key")?;

    let path = PathBuf::from(file_path);
    if !path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {:?}", path));
    }

    let doc = get_document(docs, namespace_id).await?;

    let schema_key = "schema";
    let encoded_schema_key = encode_key(schema_key.as_bytes());
    let schema_entry = doc
        .get_exact(author, encoded_schema_key.clone(), true)
        .await
        .context("Failed to get schema entry")?;

    if schema_entry.is_some() {
        return Err(anyhow::anyhow!("File import not allowed. Cannot add a file to a document with a schema."));
    }

    let encoded_key = encode_key(key.clone().as_bytes());
    let progress = doc
        .import_file(author, Bytes::from(encoded_key), &path, false)
        .await
        .context("Failed to import file")?;

    let outcome = progress
        .finish()
        .await
        .context("Failed to finish file import")?;

    Ok(ImportFileOutcome {
        hash: outcome.hash.to_string(),
        size: outcome.size,
        key: String::from_utf8(outcome.key.to_vec())
            .context("Failed to convert key to UTF-8")?,
    })
}

/// Fetches an entry from a document along with metadata like hash and timestamp.
///
/// # Parameters
/// - `docs`: Shared reference to the document store.
/// - `doc_id`: The ID of the document to fetch from.
/// - `author_id`: SS58-encoded author ID who owns the entry.
/// - `key`: Key to look up in the document.
/// - `include_empty`: Whether to return empty (tombstoned) entries.
///
/// # Returns
/// - `Some(EntryDetails)` if entry exists, else `None`.
#[derive(Serialize, Debug, Clone)]
pub struct EntryDetails {
    namespace: EntryIdDetails,
    record: RecordDetails,
}

#[derive(Serialize, Debug, Clone)]
pub struct EntryIdDetails {
    doc: String,
    key: String,
    author: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecordDetails {
    hash: String,
    len: u64,
    timestamp: u64,
}

pub async fn get_entry(
    docs: Arc<Docs<BlobStore>>,
    doc_id: String,
    author_id: String,
    key: String,
    include_empty: bool,
) -> anyhow::Result<Option<EntryDetails>> {
    let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
    let namespace_id = NamespaceId::from(namespace_id_vec);

    let author = SS58AuthorId::decode(&author_id)
        .with_context(|| format!("Failed to decode author ID {author_id}"))?;

    validate_key(&key, false)
        .await
        .context("Failed to validate key")?;

    let doc = get_document(docs, namespace_id).await?;

    let encoded_key = encode_key(key.as_bytes());
    let entry = doc
        .get_exact(author, encoded_key, include_empty)
        .await
        .context("Failed to get entry")?;

    if let Some(entry) = entry {
        let decoded_key = decode_key(entry.id().key());
        let encode_author = SS58AuthorId::from_author_id(&entry.id().author())
            .context("Failed to encode author ID")?;

        let id_details = EntryIdDetails {
            doc: entry.id().namespace().to_string(),
            key: String::from_utf8(decoded_key)
                .context("Failed to decode entry key")?,
            author: encode_author.as_ss58().to_string(),
        };

        let record_details = RecordDetails {
            hash: entry.record().content_hash().to_string(),
            len: entry.record().content_len(),
            timestamp: entry.record().timestamp(),
        };

        return Ok(Some(EntryDetails {
            namespace: id_details,
            record: record_details,
        }));
    }

    Ok(None)
}

/// Retrieves a blob entry's content using its hash.
/// 
/// # Arguments
/// * `blobs` - Shared reference to the `Blobs` store.
/// * `hash` - The hash of the blob to retrieve (as a hex string).
///
/// # Returns
/// The content of the blob as a `String`.
pub async fn get_entry_blob(
    blobs: Arc<Blobs<BlobStore>>,
    hash: String,
) -> anyhow::Result<String> {
    let hash = Hash::from_str(&hash)
        .with_context(|| format!("Failed to parse hash {hash}"))?;

    let content = get_blob_entry(blobs, hash)
        .await
        .context("Failed to get blob entry")?;

    Ok(content)
}

/// Retrieves entries from a document based on provided query parameters.
/// 
/// # Arguments
/// * `docs` - Shared reference to the `Docs` store.
/// * `doc_id` - The document ID as a string (base64-encoded).
/// * `query_params` - JSON object with optional query fields such as:
///     - `author_id`: Filter by author's SS58 address.
///     - `key`: Filter by exact key.
///     - `key_prefix`: Filter by prefix match.
///     - `limit`, `offset`: Pagination controls.
///     - `include_empty`: Include empty entries.
///     - `sort_by`: Sorting field ("author" or "key").
///     - `sort_direction`: Sorting direction ("ascending" or "descending").
///
/// # Returns
/// A list of `EntryDetails` matching the query.
pub async fn get_entries(
    docs: Arc<Docs<BlobStore>>,
    doc_id: String,
    query_params: serde_json::Value,
) -> anyhow::Result<Vec<EntryDetails>> {
    let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
    let namespace_id = NamespaceId::from(namespace_id_vec);

    let mut query = Query::all();

    if let Some(author_id_str) = query_params.get("author_id").and_then(|v| v.as_str()) {
        let author_id = SS58AuthorId::decode(author_id_str)?;
        query = query.author(author_id);
    }

    if let Some(key) = query_params.get("key").and_then(|v| v.as_str()) {
        validate_key(key, false)
            .await
            .context("Failed to validate key")?;
        let encoded_key = encode_key(key.as_bytes());
        query = query.key_exact(encoded_key);
    }

    if let Some(key_prefix) = query_params.get("key_prefix").and_then(|v| v.as_str()) {
        query = query.key_prefix(key_prefix.as_bytes());
    }

    if let Some(limit) = query_params.get("limit").and_then(|v| v.as_u64()) {
        query = query.limit(limit);
    }

    if let Some(offset) = query_params.get("offset").and_then(|v| v.as_u64()) {
        query = query.offset(offset);
    }

    if let Some(true) = query_params.get("include_empty").and_then(|v| v.as_bool()) {
        query = query.include_empty();
    }

    if let Some(sort_by) = query_params.get("sort_by").and_then(|v| v.as_str()) {
        let sort_by = match sort_by.to_lowercase().as_str() {
            "author" => SortBy::KeyAuthor,
            "key" => SortBy::AuthorKey,
            _ => {
                return Err(anyhow::anyhow!("Invalid sort_by value: {sort_by}"));
            }
        };

        if let Some(sort_direction) = query_params.get("sort_direction").and_then(|v| v.as_str()) {
            let sort_direction = match sort_direction.to_lowercase().as_str() {
                "ascending" => SortDirection::Asc,
                "descending" => SortDirection::Desc,
                _ => {
                    return Err(anyhow::anyhow!("Invalid sort_direction value: {sort_direction}"));
                }
            };
            query = query.sort_by(sort_by, sort_direction);
        } else {
            query = query.sort_by(sort_by, SortDirection::Asc);
        }
    }

    let doc = get_document(docs, namespace_id).await?;

    let mut entries = Vec::new();
    let mut entries_stream = doc
        .get_many(query)
        .await
        .with_context(|| format!("Failed to get entries for document {namespace_id}"))?;

    while let Some(entry) = entries_stream.next().await {
        let entry = entry
            .with_context(|| format!("Failed to get entry for document {namespace_id}"))?;

        let encode_author = SS58AuthorId::from_author_id(&entry.id().author())
            .context("Failed to encode author ID")?;
        let decoded_key = decode_key(entry.id().key());

        let id_details = EntryIdDetails {
            doc: entry.id().namespace().to_string(),
            key: String::from_utf8(decoded_key)
                .context("Failed to decode entry key")?,
            author: encode_author.as_ss58().to_string(),
        };
        
        let record_details = RecordDetails {
            hash: entry.record().content_hash().to_string(),
            len: entry.record().content_len(),
            timestamp: entry.record().timestamp(),
        };

        entries.push(EntryDetails {
            namespace: id_details,
            record: record_details,
        });
    }

    Ok(entries)
}

/// Deletes an entry from a document using author ID and key.
/// 
/// # Arguments
/// * `docs` - Shared reference to the `Docs` store.
/// * `doc_id` - The document ID (base64-encoded).
/// * `author_id` - SS58-encoded author ID of the entry.
/// * `key` - The key of the entry to delete.
///
/// # Returns
/// The number of deleted entries (should be 1 if successful).
pub async fn delete_entry(
    docs: Arc<Docs<BlobStore>>,
    doc_id: String,
    author_id: String,
    key: String,
) -> anyhow::Result<usize> {
    let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
    let namespace_id = NamespaceId::from(namespace_id_vec);

    let author = SS58AuthorId::decode(&author_id)
        .with_context(|| format!("Failed to decode author ID {author_id}"))?;

    validate_key(&key, true)
        .await
        .context("Failed to validate key")?;

    let doc = get_document(docs.clone(), namespace_id).await?;

    let encoded_key = encode_key(key.clone().as_bytes());
    let entry = get_entry(docs, doc_id.clone(), author_id.clone(), key.clone(), false)
        .await
        .with_context(|| format!("Failed to get entry for document {namespace_id}"))?;

    if entry.is_none() {
        return Err(anyhow::anyhow!("Entry not found for key {key}"));
    }

    let delete = doc
        .del(author, encoded_key)
        .await
        .with_context(|| format!("Failed to delete entry for document {namespace_id}"))?;

    Ok(delete)
}

/// Leaves the current document, releasing resources and closing its state.
/// 
/// # Arguments
/// * `docs` - Shared reference to the `Docs` store.
/// * `doc_id` - The document ID (base64-encoded).
///
/// # Returns
/// An empty result on success.
pub async fn leave(
    docs: Arc<Docs<BlobStore>>,
    doc_id: String,
) -> anyhow::Result<()> {
    let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
    let namespace_id = NamespaceId::from(namespace_id_vec);

    let doc = get_document(docs, namespace_id).await?;

    doc.leave()
        .await
        .with_context(|| format!("Failed to leave document {namespace_id}"))?;

    Ok(())
}

/// Retrieves the current open status of a document.
/// 
/// # Arguments
/// * `docs` - Shared reference to the `Docs` store.
/// * `doc_id` - The document ID (base64-encoded).
///
/// # Returns
/// The `OpenState` representing whether the document is joined or not.
pub async fn status (
    docs: Arc<Docs<BlobStore>>,
    doc_id: String,
) -> anyhow::Result<OpenState> {
    let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
    let namespace_id = NamespaceId::from(namespace_id_vec);

    let doc = get_document(docs, namespace_id).await?;

    let status = doc
        .status()
        .await
        .with_context(|| format!("Failed to get status of document {namespace_id}"))?;

    Ok(status)
}

/// Fetches the download policy of a document, if any.
///
/// # Arguments
/// * `docs` - Shared reference to the `Docs` store.
/// * `doc_id` - The document ID (base64-encoded).
///
/// # Returns
/// A JSON representation of the download policy.
pub async fn get_download_policy(
    docs: Arc<Docs<BlobStore>>,
    doc_id: String,
) -> anyhow::Result<serde_json::Value> {
    let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
    let namespace_id = NamespaceId::from(namespace_id_vec);

    let doc = get_document(docs, namespace_id).await?;

    let download_policy = doc
        .get_download_policy()
        .await
        .with_context(|| format!("Failed to get download policy for document {namespace_id}"))?;

    let api_policy = ApiDownloadPolicy(download_policy);

    Ok(api_policy.to_json())
}

/// Sets or updates the download policy of a document.
/// 
/// # Arguments
/// * `docs` - Shared reference to the `Docs` store.
/// * `doc_id` - The document ID (base64-encoded).
/// * `download_policy` - JSON object representing the download policy.
///
/// # Returns
/// An empty result on success.
pub async fn set_download_policy(
    docs: Arc<Docs<BlobStore>>,
    doc_id: String,
    download_policy: serde_json::Value,
) -> anyhow::Result<()> {
    let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
    let namespace_id = NamespaceId::from(namespace_id_vec);

    let doc = get_document(docs, namespace_id).await?;

    let api_policy = ApiDownloadPolicy::from_json(&download_policy)
        .with_context(|| format!("Failed to decode download policy"))?;

    doc.set_download_policy(api_policy.0)
        .await
        .with_context(|| format!("Failed to set download policy for document {namespace_id}"))?;

    Ok(())
}

// update_doc_schema
// do we need this? 

use helpers::utils::{encode_doc_id, decode_doc_id, encode_key, decode_key, SS58AuthorId, ApiDownloadPolicy, validate_key};

use iroh_blobs::net_protocol::Blobs;
use iroh_blobs::Hash;
use iroh_docs::protocol::Docs;
use iroh_blobs::store::fs::Store;
use iroh_docs::rpc::AddrInfoOptions;
use iroh_docs::{CapabilityKind, DocTicket, NamespaceId};
use iroh_docs::rpc::client::docs::{Doc, ShareMode};
use jsonschema::validator_for;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use serde_json::Value;
use bytes::Bytes;
use quic_rpc::transport::flume::FlumeConnector;
use iroh_docs::rpc::proto::{Request, Response};
use anyhow::{Result, Context};
use futures::TryStreamExt;
use futures::StreamExt;
use iroh_docs::store::{Query, SortBy, SortDirection};
use std::str::FromStr;
use serde::Serialize;
use iroh_docs::actor::OpenState;
use iroh_base::PublicKey;

/// Retrieves a document by its ID.
/// 
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
/// * `doc_id` - The unique document namespace ID.
/// 
/// # Returns
/// * `Doc` - The opened document client.
pub async fn get_document(
    docs: Arc<Docs<Store>>,
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
    blobs: Arc<Blobs<Store>>,
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
    docs: Arc<Docs<Store>>,
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
    docs: Arc<Docs<Store>>,
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
    docs: Arc<Docs<Store>>,
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
    docs: Arc<Docs<Store>>,
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
    docs: Arc<Docs<Store>>,
    ticket: String,
) -> anyhow::Result<String> {
    let doc_ticket = DocTicket::from_str(&ticket)
        .with_context(|| format!("Failed to parse document ticket {ticket}"))?;

    let doc_client = docs.client();

    let (doc, _) = doc_client
        .import_and_subscribe(doc_ticket)
        .await
        .with_context(|| "Failed to join document")?;

    Ok(doc.id().to_string())
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
    docs: Arc<Docs<Store>>,
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
    docs: Arc<Docs<Store>>,
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
    docs: Arc<Docs<Store>>,
    blobs: Arc<Blobs<Store>>,
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
    docs: Arc<Docs<Store>>,
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
    pub namespace: EntryIdDetails,
    pub record: RecordDetails,
}

#[derive(Serialize, Debug, Clone)]
pub struct EntryIdDetails {
    pub doc: String,
    pub key: String,
    pub author: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecordDetails {
    pub hash: String,
    pub len: u64,
    pub timestamp: u64,
}

pub async fn get_entry(
    docs: Arc<Docs<Store>>,
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
    blobs: Arc<Blobs<Store>>,
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
    docs: Arc<Docs<Store>>,
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
    docs: Arc<Docs<Store>>,
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
        return Err(anyhow::anyhow!("Entry not found for key '{key}'"));
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
    docs: Arc<Docs<Store>>,
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
    docs: Arc<Docs<Store>>,
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
    docs: Arc<Docs<Store>>,
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
    docs: Arc<Docs<Store>>,
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


mod tests {
    use super::*;
    use node::iroh_wrapper::{IrohNode, setup_iroh_node};
    use helpers::cli::CliArgs;
    use crate::authors::create_author;

    use anyhow::{Result, anyhow};
    use tokio::fs::{self, File};
    use std::path::PathBuf;
    use tokio::time::{sleep, Duration};
    use tokio::process::Command;
    use std::process::Stdio;
    use tokio::io::AsyncWriteExt;

    // Running tests will give any user understanding of how they should run the program in real life. 
    // step 1 is to run ```cargo run``` and fetch 'secret-key' form it and paste it in setup_node function.
    // step 2 is to run ```cargo run -- --path <path> --secret-key <your_secret_key>``` as this will create the data path and save the secret key in the data path. The test does this for user.
    // step 3 is to actually run the tests, but running it with ```cargo test``` will not work as all the tests will run in parallel and they will not be able to share the resources. Hence run the tests using ````cargo test -- --test-threads=1```.
    // If you wish to generate a lcov report, use ```cargo llvm-cov --html --tests -- --test-threads=1 --nocapture```.
    // To view the lcov file in browser, use ```open target/llvm-cov/html/index.html```.

    pub async fn setup_node() -> Result<IrohNode> {
        let secret_key = "cb9ce6327139d4d168ba753e4b12434f523221612fcabc600cdc57bba40c29de";

        fs::create_dir_all("Test").await?;

        let mut child = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--path")
        .arg("Test/test_blobs")
        .arg("--secret-key")
        .arg(secret_key)
        .stdout(Stdio::null()) // Silence output, or use `inherit()` for debug
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start cargo run");

        sleep(Duration::from_secs(5)).await;

        child.kill().await.ok();

        let args = CliArgs {
            path: Some(PathBuf::from("Test/test_blobs")),
            secret_key: Some(secret_key.to_string()), // remove this secret key
        };
        let iroh_node: IrohNode = setup_iroh_node(args).await.or_else(|_| {
            Err(anyhow!("Failed to set up Iroh node"))
        })?;
        println!("Iroh node started!");
        println!("Your NodeId: {}", iroh_node.node_id);
        Ok(iroh_node)
    }

    pub async fn delete_all_docs(
        docs: Arc<Docs<Store>>,
    ) -> Result<()> {
        let docs_list = list_docs(docs.clone()).await?;
        for (doc_id, _) in docs_list {
            let docs_clone = docs.clone(); // Clone docs again here
            drop_doc(docs_clone, doc_id.clone())
                .await
                .with_context(|| format!("Failed to drop document {doc_id}"))?;
        }

        Ok(())
    }

    // create_doc
    #[tokio::test]
    pub async fn test_create_doc() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let doc_id = create_doc(docs.clone()).await?;

        let list = list_docs(docs.clone()).await?;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, doc_id);

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // list_docs
    #[tokio::test]
    pub async fn test_list_docs() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let doc_1 = create_doc(docs.clone()).await?;
        sleep(Duration::from_secs(2)).await;
        let doc_2 = create_doc(docs.clone()).await?;

        let list = list_docs(docs.clone()).await?;
        assert_eq!(list.len(), 2);
        
        let doc_1_in_list = list.iter().any(|(id, _)| id == &doc_1);
        let doc_2_in_list = list.iter().any(|(id, _)| id == &doc_2);
        assert!(doc_1_in_list);
        assert!(doc_2_in_list);

        assert!(matches!(list[0].1, CapabilityKind::Write));
        assert!(matches!(list[1].1, CapabilityKind::Write));

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // get_document
    #[tokio::test]
    pub async fn test_get_document() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let doc_id = create_doc(docs.clone()).await?;

        let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
        let namespace_id = NamespaceId::from(namespace_id_vec);

        let doc = get_document(docs.clone(), namespace_id).await?;
        println!("Document: {:?}", doc);
        println!("{:?}", doc.id());
        println!("{}", doc.id().to_string());

        assert_eq!(doc.id(), namespace_id);

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // drop_doc
    #[tokio::test]
    pub async fn test_drop_doc() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let doc_id = create_doc(docs.clone()).await?;
        let list_before = list_docs(docs.clone()).await?;
        assert_eq!(list_before.len(), 1);

        drop_doc(docs.clone(), doc_id.clone()).await?;
        let list_after = list_docs(docs.clone()).await?;
        assert_eq!(list_after.len(), 0);

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_drop_doc_invalid_doc_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let invalid_doc_id = "not-a-valid-doc-id";

        let result = drop_doc(docs.clone(), invalid_doc_id.to_string()).await;

        let error_str = format!("{:?}", result.unwrap_err());
        assert!(
            error_str.contains("Failed to decode document ID"),
            "Expected decode error, got: {}",
            error_str
        );

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // share_doc and join_doc
    #[tokio::test]
    pub async fn test_share_doc() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let doc_id = create_doc(docs.clone()).await?;

        let ticket = share_doc(
            docs.clone(),
            doc_id.clone(),
            ShareMode::Read,
            AddrInfoOptions::Addresses,
        ).await?;

        let path_2 = Some(PathBuf::from("Test/test_blobs_1"));
        let secret_key_2 = Some("c6135803322e8c268313574920853c7f940489a74bee4d7e2566b773386283f3".to_string());
        let args = CliArgs {
            path: path_2.clone(),
            secret_key: secret_key_2,
        };
        let iroh_node_2: IrohNode = setup_iroh_node(args).await.or_else(|_| {
            Err(anyhow!("Failed to set up Iroh node"))
        })?;

        let _ = join_doc(iroh_node_2.docs.clone(), ticket).await?;

        let list_of_docs_1 = list_docs(docs.clone()).await?;
        let list_of_docs_2 = list_docs(iroh_node_2.docs.clone()).await?;

        assert_eq!(list_of_docs_1.len(), 1);
        assert_eq!(list_of_docs_2.len(), 1);
        assert_eq!(list_of_docs_1[0].0, doc_id);
        assert_eq!(list_of_docs_2[0].0, doc_id);

        // cleanup
        delete_all_docs(docs).await?;
        delete_all_docs(iroh_node_2.docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        iroh_node_2.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_share_doc_fails_on_invalid_doc_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let invalid_doc_id = "not-a-valid-doc-id";

        let result = share_doc(
            docs.clone(),
            invalid_doc_id.to_string(),
            ShareMode::Read,
            AddrInfoOptions::Addresses,
        ).await;

        let error_str = format!("{:?}", result.unwrap_err());
        assert!(
            error_str.contains("Failed to decode document ID"),
            "Expected decode error, got: {}",
            error_str
        );

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_join_doc_fails_on_invalid_ticket() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let invalid_ticket = "not-a-valid-ticket";

        let result = join_doc(docs.clone(), invalid_ticket.to_string()).await;

        let error_str = format!("{:?}", result.unwrap_err());
        assert!(
            error_str.contains("Failed to parse document ticket"),
            "Expected decode error, got: {}",
            error_str
        );

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // close_doc
    // #[tokio::test]
    // pub async fn test_close_document() -> Result<()> {
    //     let iroh_node = setup_node().await?;
    //     let docs = iroh_node.docs.clone();

    //     let doc_id = create_doc(docs.clone()).await?;

    //     sleep(Duration::from_secs(3)).await;

    //     let namespace_id_vec = decode_doc_id(&doc_id)
    //     .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
    //     let namespace_id = NamespaceId::from(namespace_id_vec);

    //     let result = get_document(docs.clone(), namespace_id.clone()).await;
    //     println!("result: {:?}", result);
        
    //     close_doc(docs.clone(), doc_id.clone()).await?;

    //     sleep(Duration::from_secs(3)).await;

    //     let result = get_document(docs.clone(), namespace_id.clone()).await;
    //     println!("result: {:?}", result);

    //     // cleanup
    //     delete_all_docs(docs).await?;
    //     fs::remove_dir_all("Test").await?;
    //     iroh_node.router.shutdown().await?;

    //     Ok(())
    // }

    // add_doc_schema
    #[tokio::test]
    pub async fn test_add_doc_schema_fails_on_invalid_doc_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author = create_author(docs.clone()).await?;

        let result = add_doc_schema(
            docs.clone(),
            author,
            "not-a-valid-doc-id".into(),
            "{}".into(),
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to decode document ID"));

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_add_doc_schema_fails_on_not_being_able_to_serialize_schema_to_json() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author = create_author(docs.clone()).await?;

        let doc_id = create_doc(docs.clone()).await?;

        let result = add_doc_schema(
            docs.clone(),
            author,
            doc_id.clone(),
            "this is not valid json".into(),
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to serialize schema to JSON"));

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_add_doc_schema_fails_on_not_being_able_to_validate_schema() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author = create_author(docs.clone()).await?;

        let doc_id = create_doc(docs.clone()).await?;

        let invalid_schema = r#"
            "this should be an object, not a string"
        "#;


        let result = add_doc_schema(
            docs.clone(),
            author,
            doc_id.clone(),
            invalid_schema.into(),
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to validate schema"));

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_add_doc_schema_fails_on_invalid_author_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let doc_id = create_doc(docs.clone()).await?;

        let valid_schema = r#"{
            "type": "object",
            "properties": {
              "owner": { "type": "string" }
            }
        }"#;

        let result = add_doc_schema(
            docs.clone(),
            "not-a-valid-author-id".into(),
            doc_id.clone(),
            valid_schema.into(),
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to decode author ID"));

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_add_doc_schema_fails_when_document_already_has_entry() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let blobs = iroh_node.blobs.clone();

        let author = create_author(docs.clone()).await?;

        let doc_id = create_doc(docs.clone()).await?;

        let _ = set_entry(docs.clone(), blobs.clone(), doc_id.clone(), author.clone(), "key".to_string(), "value".to_string()).await?;
        sleep(Duration::from_secs(3)).await;

        let valid_schema = r#"{
            "type": "object",
            "properties": {
              "owner": { "type": "string" }
            }
        }"#;

        let result = add_doc_schema(
            docs.clone(),
            author,
            doc_id.clone(),
            valid_schema.into(),
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Document already contains entries. Schema can only be added to an empty document."));

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_add_doc_schema() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author = create_author(docs.clone()).await?;

        let doc_id = create_doc(docs.clone()).await?;

        let valid_schema = r#"{
            "type": "object",
            "properties": {
              "owner": { "type": "string" },
              "name": { "type": "string" },
              "number_of_entries": { "type": "integer" },
              "terms_and_conditions": { "type": "string" }
            },
            "required": ["owner", "name", "number_of_entries", "terms_and_conditions"]
        }"#;

        let result = add_doc_schema(
            docs.clone(),
            author.clone(),
            doc_id.clone(),
            valid_schema.into(),
        ).await;

        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.is_empty());

        let schema_entry = get_entry(docs.clone(), doc_id.clone(), author.clone(), "schema".to_string(), true).await?;
        assert!(schema_entry.is_some());

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // set_entry
    #[tokio::test]
    pub async fn test_set_entry_fails_on_incorrect_namespace_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let blobs = iroh_node.blobs.clone();

        let author = create_author(docs.clone()).await?;

        let result = set_entry(
            docs.clone(),
            blobs.clone(),
            "not-a-valid-doc-id".into(),
            author.clone(),
            "key".to_string(),
            "value".to_string(),
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to decode document ID"));

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_set_entry_fails_on_incorrect_author_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let blobs = iroh_node.blobs.clone();

        let doc = create_doc(docs.clone()).await?;

        let result = set_entry(
            docs.clone(),
            blobs.clone(),
            doc.clone(),
            "not-a-valid-author-id".into(),
            "key".to_string(),
            "value".to_string(),
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to decode author ID"));

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_set_entry_fails_on_incorrect_key() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let blobs = iroh_node.blobs.clone();

        let author = create_author(docs.clone()).await?;

        let doc = create_doc(docs.clone()).await?;

        let result = set_entry(
            docs.clone(),
            blobs.clone(),
            doc.clone(),
            author.clone(),
            "schema".to_string(), // can also use "some key"
            "value".to_string(),
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to validate key"));

        // cleanup
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // not sure how to tes the two next functions as they ahve already been tested in add_doc_schema. How to regenrate it here? 
    // #[tokio::test]
    // pub async fn test_set_entry_fails_on_validating_schema_json() -> Result<()> {}

    // #[tokio::test]
    // pub async fn test_set_entry_fails_on_converting_json_to_schema() -> Result<()> {}

    #[tokio::test]
    pub async fn test_set_entry_fails_if_value_does_not_match_schema() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let blobs = iroh_node.blobs.clone();

        let author = create_author(docs.clone()).await?;

        let doc = create_doc(docs.clone()).await?;

        let namespace_id_vec = decode_doc_id(&doc)
        .with_context(|| format!("Failed to decode document ID {doc}"))?;
        let namespace_id = NamespaceId::from(namespace_id_vec);

        let valid_schema = r#"{
            "type": "object",
            "properties": {
                "owner": { "type": "string" },
                "name": { "type": "string" },
                "number_of_entries": { "type": "integer" },
                "terms_and_conditions": { "type": "string" }
            },
            "required": ["owner", "name", "number_of_entries", "terms_and_conditions"]
        }"#;

        let add_schema_result = add_doc_schema(docs.clone(), author.clone(), doc.clone(), valid_schema.to_string()).await;
        assert!(add_schema_result.is_ok());

        let valid_entry = r#"{
            "owner": "Dhiway",
            "name": "Cyra",
            "terms_and_conditions": "Agreed"
        }"#; // missing number_of_entries

        let set_entry_result = set_entry(
            docs.clone(),
            blobs.clone(),
            doc.clone(),
            author.clone(),
            "entry".to_string(),
            valid_entry.to_string(),
        ).await;
        assert!(set_entry_result.is_err());
        assert!(format!("{:?}", set_entry_result.unwrap_err()).contains("Value does not match schema"));
        
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_set_entry() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let blobs = iroh_node.blobs.clone();

        let author = create_author(docs.clone()).await?;

        let doc = create_doc(docs.clone()).await?;

        let namespace_id_vec = decode_doc_id(&doc)
        .with_context(|| format!("Failed to decode document ID {doc}"))?;
        let namespace_id = NamespaceId::from(namespace_id_vec);

        let valid_schema = r#"{
            "type": "object",
            "properties": {
                "owner": { "type": "string" },
                "name": { "type": "string" },
                "number_of_entries": { "type": "integer" },
                "terms_and_conditions": { "type": "string" }
            },
            "required": ["owner", "name", "number_of_entries", "terms_and_conditions"]
        }"#;

        let add_schema_result = add_doc_schema(docs.clone(), author.clone(), doc.clone(), valid_schema.to_string()).await;
        assert!(add_schema_result.is_ok());

        let valid_entry = r#"{
            "owner": "Dhiway",
            "name": "Cyra",
            "number_of_entries": 3,
            "terms_and_conditions": "Agreed"
        }"#;

        let set_entry_result = set_entry(
            docs.clone(),
            blobs.clone(),
            doc.clone(),
            author.clone(),
            "entry".to_string(),
            valid_entry.to_string(),
        ).await;
        assert!(set_entry_result.is_ok());

        if let Some(fetch_entry) = get_entry(docs.clone(), doc.clone(), author.clone(), "entry".to_string(), true).await? {
            assert_eq!(fetch_entry.namespace.doc, namespace_id.to_string());
            assert_eq!(fetch_entry.namespace.key, "entry".to_string());
            assert_eq!(fetch_entry.namespace.author, author.clone());
            assert_eq!(fetch_entry.record.hash, set_entry_result.unwrap());
        }
        
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // set_entry_file
    #[tokio::test]
    pub async fn test_set_entry_file_fails_on_incorrect_doc_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author = create_author(docs.clone()).await?;

        let result = set_entry_file(
            docs.clone(), 
            "not_a_valid_doc_id".to_string(), 
            author.clone(), 
            "entry".to_string(),
            "path".to_string(),
        ).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to decode document ID"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_set_entry_file_fails_on_incorrect_author_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let doc_id = create_doc(docs.clone()).await?;

        let result = set_entry_file(
            docs.clone(), 
            doc_id.clone(), 
            "not_a_valid_author_id".to_string(), 
            "entry".to_string(),
            "path".to_string(),
        ).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to decode author ID"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_set_entry_file_fails_on_incorrect_key() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author = create_author(docs.clone()).await?;

        let doc_id = create_doc(docs.clone()).await?;

        let result = set_entry_file(
            docs.clone(), 
            doc_id.clone(), 
            author.clone(), 
            "schema".to_string(), // can use 'some key' 
            "path".to_string(),
        ).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to validate key"));
        
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_set_entry_file_fails_on_non_existent_file_path() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author = create_author(docs.clone()).await?;

        let doc_id = create_doc(docs.clone()).await?;

        let result = set_entry_file(
            docs.clone(), 
            doc_id.clone(), 
            author.clone(), 
            "entry".to_string(), // can use 'some key' 
            "path".to_string(),
        ).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("File does not exist:"));
        
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_set_entry_file_fails_when_doc_already_has_schema() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author = create_author(docs.clone()).await?;

        let doc_id = create_doc(docs.clone()).await?;

        let dir = tempfile::tempdir()?;
        let file_path = dir.path().join("test_file.txt");
        let mut file = File::create(&file_path).await?;
        let data = "test data";
        file.write_all(data.as_bytes()).await?;

        let valid_schema = r#"{
            "type": "object",
            "properties": {
                "owner": { "type": "string" },
                "name": { "type": "string" },
                "number_of_entries": { "type": "integer" },
                "terms_and_conditions": { "type": "string" }
            },
            "required": ["owner", "name", "number_of_entries", "terms_and_conditions"]
        }"#;

        let add_schema_result = add_doc_schema(docs.clone(), author.clone(), doc_id.clone(), valid_schema.to_string()).await;
        sleep(Duration::from_secs(1)).await;
        assert!(add_schema_result.is_ok());

        let result = set_entry_file(
            docs.clone(), 
            doc_id.clone(), 
            author.clone(), 
            "entry".to_string(), // can use 'some key' 
            file_path.to_str().unwrap().to_string(),
        ).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("File import not allowed. Cannot add a file to a document with a schema."));

        if file_path.exists() {
            fs::remove_file(&file_path).await?;
        }
        assert!(!file_path.exists());
        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_set_entry_file() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let blobs = iroh_node.blobs.clone();

        let author = create_author(docs.clone()).await?;

        let doc_id = create_doc(docs.clone()).await?;

        let dir = tempfile::tempdir()?;
        let file_path = dir.path().join("test_file.txt");
        let mut file = File::create(&file_path).await?;
        let data = "test data";
        file.write_all(data.as_bytes()).await?;

        let result = set_entry_file(
            docs.clone(), 
            doc_id.clone(), 
            author.clone(), 
            "entry".to_string(), // can use 'some key' 
            file_path.to_str().unwrap().to_string(),
        ).await;
        assert!(result.is_ok());
        let entry_hash = result.unwrap().hash;

        let retrieved_data = get_entry_blob(blobs.clone(), entry_hash).await?;
        assert_eq!(retrieved_data, data);

        Ok(())
    }

    // get_entry
    #[tokio::test]
    pub async fn test_get_entry_fails_on_incorrect_doc_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let author = create_author(docs.clone()).await?;

        let result = get_entry(
            docs.clone(),
            "invalid-doc-id".to_string(),
            author.clone(),
            "key".to_string(),
            false,
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to decode document ID"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_get_entry_fails_on_incorrect_key() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author = create_author(docs.clone()).await?;
        let doc_id = create_doc(docs.clone()).await?;

        // Use a key that will fail validation (e.g., empty string)
        let result = get_entry(
            docs.clone(),
            doc_id.clone(),
            author.clone(),
            "".to_string(), // can not use 'some key'
            false,
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to validate key"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_get_entry_fails_on_incorrect_author_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let doc_id = create_doc(docs.clone()).await?;

        let result = get_entry(
            docs.clone(),
            doc_id.clone(),
            "invalid-author".to_string(),
            "key".to_string(),
            false,
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to decode author ID"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_get_entry_returns_nothing_when_entry_does_not_exist() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author = create_author(docs.clone()).await?;
        let doc_id = create_doc(docs.clone()).await?;

        let result = get_entry(
            docs.clone(),
            doc_id.clone(),
            author.clone(),
            "nonexistent".to_string(),
            false,
        ).await?;

        assert!(result.is_none());

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_get_entry() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let blobs = iroh_node.blobs.clone();

        let author = create_author(docs.clone()).await?;
        let doc_id = create_doc(docs.clone()).await?;
        let key = "test_key".to_string();
        let value = "test_value".to_string();

        let namespace_id_vec = decode_doc_id(&doc_id)
        .with_context(|| format!("Failed to decode document ID {doc_id}"))?;
        let namespace_id = NamespaceId::from(namespace_id_vec);


        let entry_hash = set_entry(docs.clone(), blobs.clone(), doc_id.clone(), author.clone(), key.clone(), value.clone()).await;
        assert!(entry_hash.is_ok());

        let result = get_entry(
            docs.clone(),
            doc_id.clone(),
            author.clone(),
            key.clone(),
            true,
        ).await?;

        assert!(result.is_some());
        let entry = result.unwrap();
        assert_eq!(entry.namespace.doc, namespace_id.to_string());
        assert_eq!(entry.namespace.key, key);
        assert_eq!(entry.namespace.author, author);
        assert_eq!(entry.record.hash, entry_hash.unwrap());

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    // get_entry_blob
    #[tokio::test]
    pub async fn test_get_entry_blob_fails_on_invalid_hash() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        let invalid_hash = "this is not a valid hash".to_string();

        let result = get_entry_blob(blobs.clone(), invalid_hash).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to parse hash"));

        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    // get_entries
    #[tokio::test]
    pub async fn test_get_entries_fails_on_invalid_document_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let query_params = serde_json::json!({
            "author_id": "author",
            "sort_by": "key",
            "sort_direction": "ascending"
        });

        let result = get_entries(
            docs.clone(),
            "invalid-doc-id".to_string(),
            query_params
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to decode document ID"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_get_entries_fails_on_invalid_key() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let doc_id = create_doc(docs.clone()).await?;
        let author = create_author(docs.clone()).await?;

        let query_params = serde_json::json!({
            "key": "some key",
            "author_id": author,
            "sort_by": "key",
            "sort_direction": "ascending"
        });

        let result = get_entries(
            docs.clone(),
            doc_id,
            query_params
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to validate key"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_get_entries_fails_on_invalid_sort_by_value() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let doc_id = create_doc(docs.clone()).await?;
        let author = create_author(docs.clone()).await?;

        let query_params = serde_json::json!({
            "key": "Key",
            "author_id": author,
            "sort_by": "OtherThanKeyAndAuthor",
            "sort_direction": "ascending"
        });

        let result = get_entries(
            docs.clone(),
            doc_id,
            query_params
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Invalid sort_by value:"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_get_entries_fails_on_invalid_sort_direction_value() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let doc_id = create_doc(docs.clone()).await?;
        let author = create_author(docs.clone()).await?;

        let query_params = serde_json::json!({
            "key": "Key",
            "author_id": author,
            "sort_by": "key",
            "sort_direction": "OtherThanAscendingAndDescending"
        });

        let result = get_entries(
            docs.clone(),
            doc_id,
            query_params
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Invalid sort_direction value:"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_get_entries() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let blobs = iroh_node.blobs.clone();
        let doc_id = create_doc(docs.clone()).await?;
        let author = create_author(docs.clone()).await?;

        let entr_1 = set_entry(docs.clone(), blobs.clone(), doc_id.clone(), author.clone(), "organisation_name".to_string(), "Test Org".to_string()).await?;
        let entr_2 = set_entry(docs.clone(), blobs.clone(), doc_id.clone(), author.clone(), "organisation_address".to_string(), "Test Address".to_string()).await?;
        let _ = set_entry(docs.clone(), blobs.clone(), doc_id.clone(), author.clone(), "CIN".to_string(), "00000".to_string()).await?;

        let query_params = serde_json::json!({
            "key_prefix": "org",
            "limit": "10",
            "sort_by": "key",
            "sort_direction": "ascending"
        });

        let result = get_entries(
            docs.clone(),
            doc_id.clone(),
            query_params
        ).await;

        let entries = result.unwrap();

        // assert!(result.is_ok());
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].record.hash, entr_2); // 2 is first as the order is set to ascending
        assert_eq!(entries[1].record.hash, entr_1);

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    // delete_entry
    #[tokio::test]
    pub async fn test_delete_entry_fails_on_incorrect_document_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let author = create_author(docs.clone()).await?;

        let result = delete_entry(
            docs.clone(),
            "incorrect_doc_id".to_string(),
            author.clone(),
            "Key".to_string(),
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to decode document ID"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_delete_entry_fails_on_incorrect_author_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let doc_id = create_doc(docs.clone()).await?;

        let result = delete_entry(
            docs.clone(),
            doc_id.clone(),
            "incorrect_author_id".to_string(),
            "Key".to_string(),
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to decode author ID"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_delete_entry_fails_on_incorrect_key() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let doc_id = create_doc(docs.clone()).await?;
        let author = create_author(docs.clone()).await?;

        let result = delete_entry(
            docs.clone(),
            doc_id.clone(),
            author.clone(),
            "schema".to_string(), // can use 'some key'
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to validate key"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_delete_entry_fails_if_no_match_for_entry_found() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let doc_id = create_doc(docs.clone()).await?;
        let author = create_author(docs.clone()).await?;

        let result = delete_entry(
            docs.clone(),
            doc_id.clone(),
            author.clone(),
            "Key".to_string(),
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Entry not found for key"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_delete_entry() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let blobs = iroh_node.blobs.clone();
        let author = create_author(docs.clone()).await?;
        let doc = create_doc(docs.clone()).await?;

        let entry = set_entry(
            docs.clone(),
            blobs.clone(),
            doc.clone(),
            author.clone(),
            "Key".to_string(),
            "Value".to_string(),
        ).await;
        assert!(entry.is_ok());

        sleep(Duration::from_secs(2)).await;

        let entry_before_deletion_option = get_entry(
            docs.clone(),
            doc.clone(),
            author.clone(),
            "Key".to_string(),
            true
        ).await?;
        sleep(Duration::from_secs(2)).await;

        let entry_before_deletion = entry_before_deletion_option.unwrap();
        let hash = entry.unwrap();
        assert_eq!(entry_before_deletion.record.hash, hash.clone());
        assert_ne!(entry_before_deletion.record.len, 0);
        
        let delete_result = delete_entry(
            docs.clone(),
            doc.clone(),
            author.clone(),
            "Key".to_string(),
        ).await;
        assert!(delete_result.is_ok());

        sleep(Duration::from_secs(2)).await;

        let entry_after_deletion_option = get_entry(
            docs.clone(),
            doc.clone(),
            author.clone(),
            "Key".to_string(),
            true
        ).await?;
        assert_eq!(entry_before_deletion.record.hash, hash);
        assert_eq!(entry_after_deletion_option.unwrap().record.len, 0);

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    // leave
    // not sure how to test

    // status
    // not sure how to test

    // get_download_policy and set_download_policy
    #[tokio::test]
    pub async fn test_get_download_purpose_fails_on_incorrect_document_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        
        let result = get_download_policy(
            docs.clone(), 
            "incorrect_doc_id".to_string()
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to decode document ID"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_set_download_purpose_fails_on_incorrect_document_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let download_policy = serde_json::json!({
            "policy": "nothing_except",
            "filters": []
        });
        
        let result = set_download_policy(
            docs.clone(), 
            "incorrect_doc_id".to_string(),
            download_policy
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to decode document ID"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_set_download_policy_fails_on_incorrect_download_policy_format() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let doc_id = create_doc(docs.clone()).await?;

        let incorrect_download_policy = serde_json::json!({
            "incorrect_key": "incorrect_value"
        });
        
        let result = set_download_policy(
            docs.clone(), 
            doc_id.clone(),
            incorrect_download_policy
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Failed to decode download policy"));

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_set_and_get_download_policy() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();
        let doc_id = create_doc(docs.clone()).await?;

        let download_policy = serde_json::json!({
            "policy": "nothing_except",
            "filters": []
        });
        
        let result = set_download_policy(
            docs.clone(), 
            doc_id.clone(),
            download_policy.clone()
        ).await;

        assert!(result.is_ok());

        sleep(Duration::from_secs(2)).await;

        let result = get_download_policy(
            docs.clone(), 
            doc_id.clone()
        ).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), download_policy);

        delete_all_docs(docs).await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }
}
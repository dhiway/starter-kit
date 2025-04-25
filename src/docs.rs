use iroh_blobs::net_protocol::Blobs;
use iroh_blobs::store::ExportFormat;
use iroh_blobs::Hash;
use iroh_docs::protocol::Docs;
use iroh_blobs::store::{mem::Store as BlobStore, ExportMode};
use iroh_docs::{NamespaceId, NamespaceSecret, AuthorId};
use futures::StreamExt;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde_json::Value;
use iroh_docs::rpc::client::authors;
use iroh_docs::rpc::client::docs::Doc;
use quic_rpc::transport::flume::FlumeConnector;
use iroh_docs::rpc::proto::{Request, Response};
use iroh_docs::rpc::client::docs::Client;
use iroh_docs::rpc::proto::GetExactRequest;
use bytes::Bytes;

/// Save a BTreeMap<String, Value> as a new document in iroh-docs.
pub async fn save_as_doc(
    docs: Arc<Docs<BlobStore>>,
    json: BTreeMap<String, Value>,
) -> Result<NamespaceId, Box<dyn std::error::Error>> {
    let doc_client = docs.client();
    // println!("doc_client: {:?}", doc_client);

    // let author_id = doc_client.authors().create().await?;
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

    Ok(doc.id())
}

pub async fn fetch_doc_as_json(
    docs: Arc<Docs<BlobStore>>,
    blobs: Arc<Blobs<BlobStore>>,
    doc_id: NamespaceId,
) -> Result<BTreeMap<String, Value>, Box<dyn std::error::Error>> {
    let doc_client = docs.client();

    let Some(doc) = doc_client.open(doc_id).await? else {
        return Err("Document not found".into());
    };

    let author = doc_client.authors().default().await?;
    println!("authors: {:?}", author);

    let keys = ["registry_name", "schema", "file", "archived"];

    let blob_client = blobs.client();
    let mut result_map = BTreeMap::new();

    for key in keys {
        if let Some(entry) = doc.get_exact(author, key, false).await? {
            let hash = entry.content_hash();

            // let value = blob_client
            //     .read(hash)
            //     .await?;

            // println!("value: {:?}", value);

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
use iroh::{NodeAddr, NodeId};
use iroh_blobs::{
    net_protocol::Blobs,
    rpc::client::blobs::{WrapOption, AddOutcome, BlobInfo, BlobStatus, DownloadOutcome, DownloadOptions},
    rpc::client::tags::TagInfo,
    store::mem::Store,
    util::{SetTagOption, Tag},
    store::{ExportFormat, ExportMode},
    Hash,
};
use std::{path::{Path, PathBuf}, sync::Arc};
use anyhow::{Result, Context};
use bytes::Bytes;
use futures::{StreamExt, TryStreamExt};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::str::FromStr;

/// Adds raw bytes as a blob.
/// 
/// # Arguments
/// * `blobs` - The Arc-wrapped Blobs client.
/// * `bytes` - The raw bytes to add.
/// 
/// # Returns
/// * `AddOutcome` - Metadata about the added blob.
pub async fn add_blob_bytes(
    blobs: Arc<Blobs<Store>>,
    bytes: impl Into<Bytes>,
) -> anyhow::Result<AddOutcome> {
    let blobs_client = blobs.client();
    
    let outcome = blobs_client
        .add_bytes(bytes)
        .await
        .with_context(|| "Failed to add bytes as blob")?;

    Ok(outcome)
}

/// Adds raw bytes as a blob, assigning a custom tag name.
/// 
/// # Arguments
/// * `blobs` - The Arc-wrapped Blobs client.
/// * `bytes` - The raw bytes to add.
/// * `name` - The custom name to tag the blob with.
/// 
/// # Returns
/// * `AddOutcome` - Metadata about the added blob.
pub async fn add_blob_named(
    blobs: Arc<Blobs<Store>>,
    bytes: impl Into<Bytes>,
    name: impl Into<Tag>,
) -> anyhow::Result<AddOutcome> {
    let blobs_client = blobs.client();
    
    let outcome = blobs_client
        .add_bytes_named(bytes, name)
        .await
        .with_context(|| "Failed to add named bytes as blob")?;

    Ok(outcome)
}

/// Adds a file from the filesystem as a blob.
/// 
/// # Arguments
/// * `blobs` - The Arc-wrapped Blobs client.
/// * `file_path` - The path to the file.
/// 
/// # Returns
/// * `AddOutcome` - Metadata about the added blob.
pub async fn add_blob_from_path(
    blobs: Arc<Blobs<Store>>,
    file_path: &Path
) -> anyhow::Result<AddOutcome> {
    let blobs_client = blobs.client();
    
    let abs_path = std::fs::canonicalize(file_path)
        .with_context(|| format!("Failed to canonicalize path: {:?}", file_path))?;
    
    let add_progress = blobs_client
        .add_from_path(abs_path.clone(), false, SetTagOption::Auto, WrapOption::NoWrap)
        .await
        .with_context(|| format!("Failed to add file from path: {:?}", abs_path))?;
    
    let outcome = add_progress
        .finish()
        .await
        .with_context(|| "Failed to finish blob add operation")?;

    Ok(outcome)
}

/// List blobs stored in the blob store with optional pagination.
///
/// # Arguments
///
/// * `blobs` - An `Arc` pointing to the shared `Blobs<Store>` client.
/// * `page` - The page number to retrieve (zero-based).
/// * `page_size` - The number of blobs to retrieve per page.
///
/// # Returns
/// * `Result<Vec<BlobInfo>>` - A vector of `BlobInfo` objects representing the blobs on the specified page.
/// * `anyhow::Result` - An error if the operation fails.
pub async fn list_blobs(
    blobs: Arc<Blobs<Store>>,
    page: usize,
    page_size: usize,
) -> anyhow::Result<Vec<BlobInfo>> {
    let blobs_client = blobs.client();
    
    let stream = blobs_client
        .list()
        .await
        .with_context(|| "Failed to list blobs")?;

    let blobs: Vec<BlobInfo> = stream
        .skip(page * page_size)
        .take(page_size)
        .try_collect::<Vec<_>>()
        .await
        .with_context(|| "Failed to collect blobs from stream")?;

    Ok(blobs)
}

/// Reads a blob's content by hash and returns it as a UTF-8 string or base64-encoded string if binary.
/// 
/// # Arguments
/// * `blobs` - The Arc-wrapped Blobs client.
/// * `hash` - The hash identifying the blob.
/// 
/// # Returns
/// * `String` - UTF-8 content or base64-encoded blob data.
pub async fn get_blob(
    blobs: Arc<Blobs<Store>>,
    hash: String,
) -> anyhow::Result<String> {
    let blobs_client = blobs.client();

    let hash = Hash::from_str(&hash)
        .with_context(|| format!("Failed to parse hash: {}", hash))?;
    
    let blob_content = blobs_client
        .read_to_bytes(hash)
        .await
        .with_context(|| format!("Failed to read blob with hash: {}", hash))?;

    match String::from_utf8(blob_content.to_vec()) {
        Ok(utf8_string) => Ok(utf8_string),
        Err(_) => {
            let base64_string = STANDARD.encode(blob_content);
            Ok(base64_string)
        },
    }
}

/// Gets the current status of a blob by its hash (e.g., NotFound, Partial, Complete).
/// 
/// # Arguments
/// * `blobs` - The Arc-wrapped Blobs client.
/// * `hash` - The hash identifying the blob.
/// 
/// # Returns
/// * `String` - Blob status as a string.
pub async fn status_blob(
    blobs: Arc<Blobs<Store>>,
    hash: String,
) -> anyhow::Result<String> {
    let blobs_client = blobs.client();

    let hash = Hash::from_str(&hash)
        .with_context(|| format!("Failed to parse hash: {}", hash))?;

    let blob_status = blobs_client
        .status(hash)
        .await
        .with_context(|| format!("Failed to get status for blob with hash: {}", hash))?;

    let status_string = match blob_status {
        BlobStatus::NotFound => "NotFound".to_string(),
        BlobStatus::Partial { size: _ }=> "Partial".to_string(),
        BlobStatus::Complete {size: _  }=> "Complete".to_string(),
    };

    Ok(status_string)
}

/// Checks if a blob with the given hash exists locally.
/// 
/// # Arguments
/// * `blobs` - The Arc-wrapped Blobs client.
/// * `hash` - The hash to check for existence.
/// 
/// # Returns
/// * `bool` - True if blob exists, false otherwise.
pub async fn has_blob(
    blobs: Arc<Blobs<Store>>,
    hash: String,
) -> anyhow::Result<bool> {
    let blobs_client = blobs.client();

    let hash = Hash::from_str(&hash)
        .with_context(|| format!("Failed to parse hash: {}", hash))?;

    let is_present = blobs_client
        .has(hash)
        .await
        .with_context(|| format!("Failed to check if blob with hash: {} is present", hash))?;

    Ok(is_present)
}

/// Downloads a blob from a specified node.
/// 
/// # Arguments
/// * `blobs` - The Arc-wrapped Blobs client.
/// * `hash` - The hash of the blob to download.
/// * `node_id` - The node ID to download the blob from.
/// 
/// # Returns
/// * `DownloadOutcome` - Result of the download operation.
pub async fn download_blob(
    blobs: Arc<Blobs<Store>>,
    hash: String,
    node_id: String,
) -> anyhow::Result<DownloadOutcome> {
    let blobs_client = blobs.client();

    let hash = Hash::from_str(&hash)
        .with_context(|| format!("Failed to parse hash: {}", hash))?;

    let node_id = NodeId::from_str(&node_id)
        .with_context(|| format!("Failed to parse node ID: {}", node_id))?;

    let node_addr = NodeAddr::from(node_id);

    let download_progress = blobs_client
        .download(hash, node_addr)
        .await
        .with_context(|| format!("Failed to initiate download for blob with hash: {}", hash))?;

    let download_outcome = download_progress
        .finish()
        .await
        .with_context(|| format!("Failed to finish download for blob with hash: {}", hash))?;

    Ok(download_outcome)
}


/// Downloads a sequence of hashes from a specified node.
/// 
/// # Arguments
/// * `blobs` - The Arc-wrapped Blobs client.
/// * `hashes` - The sequence of hashes to download.
/// * `node_id` - The node ID to download the hashes from.
/// 
/// # Returns
/// * `DownloadOutcome` - Result of the download operation.
pub async fn download_hash_sequence(
    blobs: Arc<Blobs<Store>>,
    hash: String,
    node_id: String,
) -> anyhow::Result<DownloadOutcome> {
    let blobs_client = blobs.client();

    let hash = Hash::from_str(&hash)
        .with_context(|| format!("Failed to parse hash sequence: {}", hash))?;

    let node_id = NodeId::from_str(&node_id)
        .with_context(|| format!("Failed to parse node ID: {}", node_id))?;

    let node_addr = NodeAddr::from(node_id);

    let download_progress = blobs_client
        .download_hash_seq(hash, node_addr)
        .await
        .with_context(|| format!("Failed to initiate hash sequence download with hash: {}", hash))?;

    let download_outcome = download_progress
        .finish()
        .await
        .with_context(|| format!("Failed to finish hash sequence download with hash: {}", hash))?;

    Ok(download_outcome)
}

/// Downloads a blob with custom download options.
/// 
/// # Arguments
/// * `blobs` - The Arc-wrapped Blobs client.
/// * `hash` - The hash of the blob to download.
/// * `options` - Custom download options to apply.
/// 
/// # Returns
/// * `DownloadOutcome` - Result of the download operation.
pub async fn download_with_options(
    blobs: Arc<Blobs<Store>>,
    hash: String,
    options: DownloadOptions,
) -> anyhow::Result<DownloadOutcome> {
    let blobs_client = blobs.client();

    let hash = Hash::from_str(&hash)
        .with_context(|| format!("Failed to parse hash with options: {}", hash))?;

    let download_progress = blobs_client
        .download_with_opts(hash, options)
        .await
        .with_context(|| format!("Failed to initiate download with options for blob with hash: {}", hash))?;

    let download_outcome = download_progress
        .finish()
        .await
        .with_context(|| format!("Failed to finish download with options for blob with hash: {}", hash))?;

    Ok(download_outcome)
}

/// Lists all available tags.
/// 
/// # Arguments
/// * `blobs` - The Arc-wrapped Blobs client.
/// 
/// # Returns
/// * `Vec<TagInfo>` - A list of tag metadata.
pub async fn list_tags(
    blobs: Arc<Blobs<Store>>,
) -> anyhow::Result<Vec<TagInfo>> {
    let blobs_client = blobs.client();

    let tag_client = blobs_client.tags();

    let stream = tag_client
        .list()
        .await
        .with_context(|| "Failed to list tags")?;

    let tags: Vec<TagInfo> = stream
        .try_collect::<Vec<_>>()
        .await
        .with_context(|| "Failed to collect tags from stream")?;

    Ok(tags)
}

/// Deletes a specific tag.
/// 
/// # Arguments
/// * `blobs` - The Arc-wrapped Blobs client.
/// * `tag_name` - The name of the tag to delete.
/// 
/// # Returns
/// * `()` - Empty result on success.
pub async fn delete_tag(
    blobs: Arc<Blobs<Store>>,
    tag_name: impl AsRef<[u8]>,
) -> anyhow::Result<()> {
    let blobs_client = blobs.client();

    let tag_client = blobs_client.tags();

    let tag = Tag(Bytes::copy_from_slice(tag_name.as_ref()));

    tag_client
        .delete(tag.clone())
        .await
        .with_context(|| format!("Failed to delete tag: {}", tag))?;

    Ok(())
}

/// Exports a blob to a file on disk.
/// 
/// # Arguments
/// * `blobs` - The Arc-wrapped Blobs client.
/// * `hash` - The hash of the blob to export.
/// * `destination` - The file path where the blob should be saved.
/// 
/// # Returns
/// * `()` - Empty result on success.
pub async fn export_blob_to_file(
    blobs: Arc<Blobs<Store>>,
    hash: String,
    destination: PathBuf,
) -> Result<()> {
    let blobs_client = blobs.client();

    let hash = Hash::from_str(&hash)
        .with_context(|| format!("Failed to parse hash for export: {}", hash))?;

    blobs_client
        .export(hash, destination.clone() , ExportFormat::Blob, ExportMode::Copy)
        .await?
        .finish()
        .await?;
    println!("Exported blob with hash: {} to file: {:?}", hash, destination);

    Ok(())
}

// delete_blob
// do we need this?
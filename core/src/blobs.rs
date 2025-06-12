use iroh::{NodeAddr, NodeId};
use iroh_blobs::{
    net_protocol::Blobs,
    rpc::client::blobs::{WrapOption, AddOutcome, BlobInfo, BlobStatus, DownloadOutcome, DownloadOptions},
    rpc::client::tags::TagInfo,
    store::fs::Store,
    util::{SetTagOption, Tag},
    store::{ExportFormat, ExportMode},
    Hash,
};
use std::{path::{Path, PathBuf}, sync::Arc, fmt};
use anyhow::{Result, Context};
use bytes::Bytes;
use futures::{StreamExt, TryStreamExt};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::str::FromStr;

// Errors
#[derive(Debug, PartialEq, Clone)]
pub enum BlobError {
    // /// The specified blob was not found in the store.
    // BlobNotFound,
    /// The provided blob hash format is invalid or cannot be decoded.
    InvalidBlobHashFormat,
    /// Failed to add bytes as a blob.
    FailedToAddBlobBytes,
    /// Failed to add named bytes as a blob.
    FailedToAddNamedBlob,
    /// Failed to add a blob from the specified file path.
    FailedToAddBlobFromPath,
    /// Failed to canonicalize the provided file path.
    FailedToCanonicalizePath,
    /// Failed to finish the blob add operation.
    FailedToFinishBlobAdd,
    /// Failed to list blobs from the store.
    FailedToListBlobs,
    /// Failed to collect blobs from the stream.
    FailedToCollectBlobs,
    /// Failed to read the blob content.
    FailedToReadBlob,
    /// Failed to get the status of the blob.
    FailedToGetBlobStatus,
    /// Failed to check if the blob exists.
    FailedToCheckBlobExistence,
    /// Failed to initiate blob download.
    FailedToInitiateDownload,
    /// Failed to finish blob download.
    FailedToFinishDownload,
    /// Failed to parse the node ID.
    InvalidNodeIdFormat,
    /// Failed to initiate hash sequence download.
    FailedToInitiateHashSequenceDownload,
    /// Failed to finish hash sequence download.
    FailedToFinishHashSequenceDownload,
    /// Failed to initiate download with options.
    FailedToInitiateDownloadWithOptions,
    /// Failed to finish download with options.
    FailedToFinishDownloadWithOptions,
    /// Failed to list tags.
    FailedToListTags,
    /// Failed to collect tags from the stream.
    FailedToCollectTags,
    /// Failed to delete the specified tag.
    FailedToDeleteTag,
    /// Failed to export the blob to a file.
    FailedToExportBlob,
    /// Failed to finish the blob export operation.
    FailedToFinishExportBlob,
    // /// The export destination path is invalid or cannot be canonicalized.
    // InvalidExportDestination,
}

impl fmt::Display for BlobError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for BlobError {}


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
) -> Result<AddOutcome, BlobError> {
    let blobs_client = blobs.client();
    
    let outcome = blobs_client
        .add_bytes(bytes)
        .await
        .map_err(|_| BlobError::FailedToAddBlobBytes)?;

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
) -> Result<AddOutcome, BlobError> {
    let blobs_client = blobs.client();
    
    let outcome = blobs_client
        .add_bytes_named(bytes, name)
        .await
        .map_err(|_| BlobError::FailedToAddNamedBlob)?;

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
) -> Result<AddOutcome, BlobError> {
    let blobs_client = blobs.client();
    
    let abs_path = std::fs::canonicalize(file_path)
        .map_err(|_| BlobError::FailedToCanonicalizePath)?;
    
    let add_progress = blobs_client
        .add_from_path(abs_path.clone(), false, SetTagOption::Auto, WrapOption::NoWrap)
        .await
        .map_err(|_| BlobError::FailedToAddBlobFromPath)?;
    
    let outcome = add_progress
        .finish()
        .await
        .map_err(|_| BlobError::FailedToFinishBlobAdd)?;

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
) -> Result<Vec<BlobInfo>, BlobError> {
    let blobs_client = blobs.client();
    
    let stream = blobs_client
        .list()
        .await
        .map_err(|_| BlobError::FailedToListBlobs)?;

    let blobs: Vec<BlobInfo> = stream
        .skip(page * page_size)
        .take(page_size)
        .try_collect::<Vec<_>>()
        .await
        .map_err(|_| BlobError::FailedToCollectBlobs)?;

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
) -> Result<String, BlobError> {
    let blobs_client = blobs.client();

    let hash = Hash::from_str(&hash)
        .map_err(|_| BlobError::InvalidBlobHashFormat)?;
    
    let blob_content = blobs_client
        .read_to_bytes(hash)
        .await
        .map_err(|_| BlobError::FailedToReadBlob)?;

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
) -> Result<String, BlobError> {
    let blobs_client = blobs.client();

    let hash = Hash::from_str(&hash)
        .map_err(|_| BlobError::InvalidBlobHashFormat)?;

    let blob_status = blobs_client
        .status(hash)
        .await
        .map_err(|_| BlobError::FailedToGetBlobStatus)?;

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
) -> Result<bool, BlobError> {
    let blobs_client = blobs.client();

    let hash = Hash::from_str(&hash)
        .map_err(|_| BlobError::InvalidBlobHashFormat)?;

    let is_present = blobs_client
        .has(hash)
        .await
        .map_err(|_| BlobError::FailedToCheckBlobExistence)?;

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
) -> Result<DownloadOutcome, BlobError> {
    let blobs_client = blobs.client();

    let hash = Hash::from_str(&hash)
        .map_err(|_| BlobError::InvalidBlobHashFormat)?;

    let node_id = NodeId::from_str(&node_id)
        .map_err(|_| BlobError::InvalidNodeIdFormat)?;

    let node_addr = NodeAddr::from(node_id);

    let download_progress = blobs_client
        .download(hash, node_addr)
        .await
        .map_err(|_| BlobError::FailedToInitiateDownload)?;

    let download_outcome = download_progress
        .finish()
        .await
        .map_err(|_| BlobError::FailedToFinishDownload)?;

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
) -> Result<DownloadOutcome, BlobError> {
    let blobs_client = blobs.client();

    let hash = Hash::from_str(&hash)
        .map_err(|_| BlobError::InvalidBlobHashFormat)?;

    let node_id = NodeId::from_str(&node_id)
        .map_err(|_| BlobError::InvalidNodeIdFormat)?;

    let node_addr = NodeAddr::from(node_id);

    let download_progress = blobs_client
        .download_hash_seq(hash, node_addr)
        .await
        .map_err(|_| BlobError::FailedToInitiateHashSequenceDownload)?;

    let download_outcome = download_progress
        .finish()
        .await
        .map_err(|_| BlobError::FailedToFinishHashSequenceDownload)?;

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
) -> Result<DownloadOutcome, BlobError> {
    let blobs_client = blobs.client();

    let hash = Hash::from_str(&hash)
        .map_err(|_| BlobError::InvalidBlobHashFormat)?;

    let download_progress = blobs_client
        .download_with_opts(hash, options)
        .await
        .map_err(|_| BlobError::FailedToInitiateDownloadWithOptions)?;

    let download_outcome = download_progress
        .finish()
        .await
        .map_err(|_| BlobError::FailedToFinishDownloadWithOptions)?;

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
) -> Result<Vec<TagInfo>, BlobError> {
    let blobs_client = blobs.client();

    let tag_client = blobs_client.tags();

    let stream = tag_client
        .list()
        .await
        .map_err(|_| BlobError::FailedToListTags)?;

    let tags: Vec<TagInfo> = stream
        .try_collect::<Vec<_>>()
        .await
        .map_err(|_| BlobError::FailedToCollectTags)?;

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
) -> Result<(), BlobError> {
    let blobs_client = blobs.client();

    let tag_client = blobs_client.tags();

    let tag = Tag(Bytes::copy_from_slice(tag_name.as_ref()));

    tag_client
        .delete(tag.clone())
        .await
        .map_err(|_| BlobError::FailedToDeleteTag)?;

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
) -> Result<(), BlobError> {
    let blobs_client = blobs.client();

    let hash = Hash::from_str(&hash)
        .map_err(|_| BlobError::InvalidBlobHashFormat)?;

    blobs_client
        .export(hash, destination.clone() , ExportFormat::Blob, ExportMode::Copy)
        .await
        .map_err(|_| BlobError::FailedToExportBlob)?
        .finish()
        .await
        .map_err(|_| BlobError::FailedToFinishExportBlob)?;

    Ok(())
}

// delete_blob
// do we need this?

#[cfg(test)]
mod tests {
    use super::*;
    use node::iroh_wrapper::{
        setup_iroh_node,
        IrohNode};
    use helpers::cli::CliArgs;

    use anyhow::{anyhow, Result};
    use tokio::fs::{self, File};
    use tokio::io::AsyncWriteExt;
    use tokio::time::{sleep, Duration};
    use tokio::task::JoinHandle;    
    use tempfile::tempdir;
    use std::path::PathBuf;
    use tokio::process::Command;
    use std::process::Stdio;

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

    // add_blob_bytes
    #[tokio::test]
    pub async fn test_add_blob_bytes() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();
        let bytes = Bytes::from("Unit test");
        
        let outcome = add_blob_bytes(blobs.clone(), bytes).await?;
        
        let output_string = get_blob(blobs, outcome.hash.to_string()).await?;
        assert_eq!(output_string, "Unit test");

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // add_blob_named
    #[tokio::test]
    pub async fn test_add_blob_named() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();
        let bytes = Bytes::from("Unit test");
        let tag_name = "test_tag";

        let outcome = add_blob_named(blobs.clone(), bytes, tag_name).await?;
        let output_string = get_blob(blobs, outcome.hash.to_string()).await?;
        assert_eq!(output_string, "Unit test");
        assert_eq!(outcome.tag, tag_name.into());

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // add_blob_from_path
    #[tokio::test]
    pub async fn test_add_blob_from_path() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        fs::create_dir_all("Test").await?;
        let mut file = File::create("Test/dummy_blob_data.txt").await?;
        file.write_all(b"This is a file with dummy blob data.").await?;

        let outcome = add_blob_from_path(blobs.clone(), &PathBuf::from("Test/dummy_blob_data.txt")).await?;
        let output_string = get_blob(blobs, outcome.hash.to_string()).await?;
        assert_eq!(output_string, "This is a file with dummy blob data.");

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_file("Test/dummy_blob_data.txt").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_add_blob_from_path_fails_on_invalid_path() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        let invalid_path = Path::new("non_existent_file.txt");

        let result = add_blob_from_path(blobs, invalid_path).await;

        assert!(matches!(result, Err(BlobError::FailedToCanonicalizePath)));

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }


    // list_blobs
    #[tokio::test]
    pub async fn test_list_blobs() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();
        let page = 0;
        let page_size = 10;

        // Add some blobs for testing
        let bytes_1 = Bytes::from("Blob data 1");
        let outcome_1 = add_blob_bytes(blobs.clone(), bytes_1).await?;
        
        let bytes_2 = Bytes::from("Blob data 2");
        let outcome_2 = add_blob_bytes(blobs.clone(), bytes_2).await?;

        // List blobs
        let blobs_list = list_blobs(blobs.clone(), page, page_size).await?;
        assert_eq!(blobs_list.len(), 2);
        assert_eq!(blobs_list[0].hash, outcome_1.hash);
        assert_eq!(blobs_list[1].hash, outcome_2.hash);

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_list_blobs_fails_on_invalid_stream() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        // Insert some blob so list has something to work with (optional)
        let _ = add_blob_bytes(blobs.clone(), Bytes::from("Sample data")).await?;

        // Drop the router to simulate client failure (stream will break)
        iroh_node.router.shutdown().await?;

        let result = list_blobs(blobs.clone(), 0, 10).await;

        assert!(matches!(result, Err(BlobError::FailedToCollectBlobs)));

        // Clean up
        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // get_blob
    #[tokio::test]
    pub async fn test_get_blob() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();
        let bytes = Bytes::from("Unit test");
        
        let outcome = add_blob_bytes(blobs.clone(), bytes).await?;
        let output_string = get_blob(blobs, outcome.hash.to_string()).await?;
        assert_eq!(output_string, "Unit test");

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_get_blob_fails_on_invalid_hash() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        let invalid_hash = "invalid-hash-value".to_string();

        let result = get_blob(blobs, invalid_hash.clone()).await;

        assert!(matches!(result, Err(BlobError::InvalidBlobHashFormat)));

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_get_blob_returns_base64_on_invalid_utf8() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        // Add non-UTF8 binary data
        let non_utf8_bytes = Bytes::from(vec![0xff, 0xfe, 0xfd, 0xfc]);
        let add_outcome = add_blob_bytes(blobs.clone(), non_utf8_bytes.clone()).await?;
        let hash_str = add_outcome.hash.to_string();

        let result = get_blob(blobs.clone(), hash_str).await?;

        // This should NOT be UTF-8 decodable and should return base64
        let expected_base64 = base64::engine::general_purpose::STANDARD.encode(non_utf8_bytes);
        assert_eq!(result, expected_base64);

        // Clean up
        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    // status_blob
    #[tokio::test]
    pub async fn test_status_blob() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();
        let bytes = Bytes::from("Complete test");
        
        let outcome_complete = add_blob_bytes(blobs.clone(), bytes).await?;
        let blob_status = status_blob(blobs.clone(), outcome_complete.hash.to_string()).await?;
        assert_eq!(blob_status, "Complete");

        // replacing last 8 characters of the hash with 'notfound' to generate incorrect hash
        let fake_hash = Hash::from_bytes([1u8; 32]);
        let blob_status = status_blob(blobs.clone(), fake_hash.to_string()).await?;
        assert_eq!(blob_status, "NotFound");

        // try to simulate Partial status
        // NOTE: we are trying to spin up second node, hence change the path and the secret key form node 1.
        let path_2 = Some(PathBuf::from("Test/test_blobs_1"));
        let secret_key_2 = Some("c6135803322e8c268313574920853c7f940489a74bee4d7e2566b773386283f3".to_string());
        let args = CliArgs {
            path: path_2.clone(),
            secret_key: secret_key_2,
        };
        let iroh_node_2: IrohNode = setup_iroh_node(args).await.or_else(|_| {
            Err(anyhow!("Failed to set up Iroh node"))
        })?;

        let dir = tempdir()?;
        let file_path = dir.path().join("large_file.txt");
        let mut file = File::create(&file_path).await?;
        let data = vec![0u8; 100 * 1024 * 1024]; // 100MB of zeros(A rather large file)
        file.write_all(&data).await?;

        let outcome = add_blob_from_path(iroh_node_2.blobs.clone(), &file_path).await?;

        let blobs_clone = blobs.clone();
        let download_handle: JoinHandle<anyhow::Result<DownloadOutcome>> = tokio::spawn(async move {
            download_blob(
                blobs_clone, 
                outcome.hash.clone().to_string(), 
                iroh_node_2.node_id.clone().to_string()
            )
            .await
            .map_err(|e| anyhow!("Download failed: {}", e))
        });
        sleep(Duration::from_secs(2)).await;
        download_handle.abort();

        let blob_status = status_blob(blobs.clone(), outcome.hash.clone().to_string()).await?;
        // aborting the download process will not return 'Partial' status, it will return 'NotFound'
        assert_ne!(blob_status, "Partial");
        assert_eq!(blob_status, "NotFound");

        fs::remove_dir_all(dir).await?;
        fs::remove_dir_all("Test/test_blobs").await?;
        if let Some(path_to_remove) = path_2 {
            fs::remove_dir_all(path_to_remove).await?;
        }
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        iroh_node_2.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_status_blob_fails_on_invalid_hash() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        let invalid_hash = "invalid-hash".to_string();

        let result = status_blob(blobs, invalid_hash.clone()).await;

        assert!(matches!(result, Err(BlobError::InvalidBlobHashFormat)));

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }


    // has_blob
    #[tokio::test]
    pub async fn test_has_blob() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();
        let bytes = Bytes::from("Unit test");
        
        let outcome = add_blob_bytes(blobs.clone(), bytes).await?;
        let blob_exists = has_blob(blobs.clone(), outcome.hash.to_string()).await?;
        assert_eq!(blob_exists, true);

        let fake_hash = Hash::from_bytes([1u8; 32]);
        let blob_exists = has_blob(blobs, fake_hash.to_string()).await?;
        assert_eq!(blob_exists, false);

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_has_blob_fails_on_invalid_hash() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        let invalid_hash = "invalid-hash-value".to_string();

        let result = has_blob(blobs, invalid_hash.clone()).await;

        assert!(matches!(result, Err(BlobError::InvalidBlobHashFormat)));

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    // download_blob
    #[tokio::test]
    pub async fn test_download_blob() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();
        let bytes = Bytes::from("Unit test");
        
        let outcome = add_blob_bytes(blobs.clone(), bytes).await?;
        let node_id = iroh_node.node_id.clone().to_string();
        
        // Simulate a different node ID for download
        let path_2 = Some(PathBuf::from("Test/test_blobs_1"));
        let secret_key_2 = Some("c6135803322e8c268313574920853c7f940489a74bee4d7e2566b773386283f3".to_string());
        let args = CliArgs {
            path: path_2.clone(),
            secret_key: secret_key_2,
        };
        let iroh_node_2: IrohNode = setup_iroh_node(args).await.or_else(|_| {
            Err(anyhow!("Failed to set up Iroh node"))
        })?;
        
        let _ = download_blob(iroh_node_2.blobs.clone(), outcome.hash.to_string(), node_id).await?;
        let get_blob = get_blob(iroh_node_2.blobs.clone(), outcome.hash.to_string()).await?;
        assert_eq!(get_blob, "Unit test");

        fs::remove_dir_all("Test/test_blobs").await?;
        if let Some(path_to_remove) = path_2 {
            fs::remove_dir_all(path_to_remove).await?;
        }
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        iroh_node_2.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_download_blob_fails_on_invalid_hash() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        let invalid_hash = "bad-hash";
        let valid_node_id = iroh_node.node_id.to_string();

        let result = download_blob(blobs, invalid_hash.to_string(), valid_node_id).await;

        assert!(matches!(result, Err(BlobError::InvalidBlobHashFormat)));

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_download_blob_fails_on_invalid_node_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        // Add a valid blob
        let blob_bytes = Bytes::from("sample");
        let add_outcome = add_blob_bytes(blobs.clone(), blob_bytes).await?;
        let valid_hash = add_outcome.hash.to_string();

        let invalid_node_id = "bad-node-id";

        let result = download_blob(blobs, valid_hash, invalid_node_id.to_string()).await;

        assert!(matches!(result, Err(BlobError::InvalidNodeIdFormat)));

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    // not sure why this doesn't work
    // download_hash_sequence
    // #[tokio::test]
    // pub async fn test_download_hash_sequence() -> Result<()>{
    //     let iroh_node = setup_node().await?;
    //     let blobs = iroh_node.blobs.clone();
    //     let bytes = Bytes::from("Unit test");

    //     let outcome = add_blob_bytes(blobs.clone(), bytes).await?;
    //     let node_id = iroh_node.node_id.clone().to_string();

    //     sleep(Duration::from_secs(3)).await;

    //     // call has_blob on iroh_node to ensure it is in place
    //     let has_blob = has_blob(blobs.clone(), outcome.hash.to_string()).await?;
    //     println!("Has blob: {:?}", has_blob);
    //     assert_eq!(has_blob, true);

    //     sleep(Duration::from_secs(3)).await;

    //     let path_2 = Some(PathBuf::from("Test/test_blobs_1"));
    //     let secret_key_2 = Some("c6135803322e8c268313574920853c7f940489a74bee4d7e2566b773386283f3".to_string());
    //     let args = CliArgs {
    //         path: path_2.clone(),
    //         secret_key: secret_key_2,
    //     };
    //     let iroh_node_2: IrohNode = setup_iroh_node(args).await.or_else(|_| {
    //         Err(anyhow!("Failed to set up Iroh node"))
    //     })?;

    //     sleep(Duration::from_secs(10)).await;

    //     let download_outcome = download_hash_sequence(iroh_node_2.blobs.clone(), outcome.hash.to_string(), node_id).await?;
    //     println!("Download outcome: {:?}", download_outcome);

    //     Ok(())
    // }

    #[tokio::test]
    pub async fn test_download_hash_sequence_fails_on_invalid_hash() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        let invalid_hash = "bad-hash";
        let valid_node_id = iroh_node.node_id.to_string();

        let result = download_hash_sequence(blobs, invalid_hash.to_string(), valid_node_id).await;
        let error_str = format!("{:?}", result.clone().unwrap_err());

        assert!(matches!(result, Err(BlobError::InvalidBlobHashFormat)));

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn test_download_hash_sequence_fails_on_invalid_node_id() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        let bytes = Bytes::from("test");
        let blob_outcome = add_blob_bytes(blobs.clone(), bytes).await?;
        let valid_hash = blob_outcome.hash.to_string();
        let invalid_node_id = "not-a-node-id";

        let result = download_hash_sequence(blobs, valid_hash, invalid_node_id.to_string()).await;
        let error_str = format!("{:?}", result.clone().unwrap_err());

        assert!(matches!(result, Err(BlobError::InvalidNodeIdFormat)));

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    // download_with_options
    #[tokio::test]
    pub async fn test_download_with_options() -> Result<()> {
        let iroh_node = setup_node().await?;

        // setup node 2
        let path_2 = Some(PathBuf::from("Test/test_blobs_1"));
        let secret_key_2 = Some("c6135803322e8c268313574920853c7f940489a74bee4d7e2566b773386283f3".to_string());
        let args_2 = CliArgs {
            path: path_2.clone(),
            secret_key: secret_key_2,
        };
        let iroh_node_2: IrohNode = setup_iroh_node(args_2).await.or_else(|_| {
            Err(anyhow!("Failed to set up Iroh node 2"))
        })?;

        let data = String::from("Blob data 2");
        let bytes = Bytes::from(data.clone());  // clone ensures data stays alive
        let outcome = add_blob_bytes(iroh_node_2.blobs.clone(), bytes).await?;

        // setup node 3
        let path_3 = Some(PathBuf::from("Test/test_blobs_2"));
        let secret_key_3 = Some("c6135803322e8c268313574920853c7f940489a74bee4d7e2566b773386283f4".to_string());
        let args_3 = CliArgs {
            path: path_3.clone(),
            secret_key: secret_key_3,
        };
        let iroh_node_3: IrohNode = setup_iroh_node(args_3).await.or_else(|_| {
            Err(anyhow!("Failed to set up Iroh node 3"))
        })?;

        let download_options: DownloadOptions = DownloadOptions {
            format: iroh_blobs::BlobFormat::Raw,
            nodes: vec![
                iroh::NodeAddr::from(iroh_node.node_id), 
                iroh::NodeAddr::from(iroh_node_2.node_id)
            ],
            tag: SetTagOption::Auto,
            mode: iroh_blobs::net_protocol::DownloadMode::Direct,
        };

        let download_outcome = download_with_options(iroh_node_3.blobs, outcome.hash.to_string(), download_options).await?;
        assert_eq!(download_outcome.downloaded_size, "Blob data 2".len() as u64); 

        // Clean up
        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test/test_blobs_1").await?;
        fs::remove_dir_all("Test/test_blobs_2").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        iroh_node_2.router.shutdown().await?;
        iroh_node_3.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_download_with_options_fails_on_invalid_hash() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        let invalid_hash = "not-a-valid-hash".to_string();
        let download_options: DownloadOptions = DownloadOptions {
            format: iroh_blobs::BlobFormat::Raw,
            nodes: vec![
                iroh::NodeAddr::from(iroh_node.node_id)
            ],
            tag: SetTagOption::Auto,
            mode: iroh_blobs::net_protocol::DownloadMode::Direct,
        };

        let result = download_with_options(blobs, invalid_hash.clone(), download_options).await;

        assert!(matches!(result, Err(BlobError::InvalidBlobHashFormat)));

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

    // list_tags
    #[tokio::test]
    pub async fn test_list_tags() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        let bytes_1 = Bytes::from("Unit test 1");
        let tag_name_1 = "Tag 1";
        let _ = add_blob_named(blobs.clone(), bytes_1, tag_name_1).await?;

        let bytes_2 = Bytes::from("Unit test 2");
        let tag_name_2 = "Tag 2";
        let _ = add_blob_named(blobs.clone(), bytes_2, tag_name_2).await?;

        let tags = list_tags(blobs).await?;
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, tag_name_1.into());
        assert_eq!(tags[1].name, tag_name_2.into());

        // Clean up
        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // #[tokio::test]
    // pub async fn test_list_tags_fails_on_broken_stream() -> Result<()> {
    //     let iroh_node = setup_node().await?;
    //     let blobs = iroh_node.blobs.clone();

    //     let bytes_1 = Bytes::from("Unit test 1");
    //     let tag_name_1 = "Tag 1";
    //     let _ = add_blob_named(blobs.clone(), bytes_1, tag_name_1).await?;

    //     // Kill the node before consuming the stream
    //     iroh_node.router.shutdown().await?;

    //     sleep(Duration::from_millis(500)).await;

    //     let result = list_tags(blobs.clone()).await;
    //     let error_str = format!("{:?}", result.unwrap_err());

    //     assert!(
    //         error_str.contains("Failed to collect tags from stream"),
    //         "Expected tag collection failure, got: {}",
    //         error_str
    //     );

    //     Ok(())
    // }


    // delete_tag
    #[tokio::test]
    pub async fn test_delete_tag() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();

        let bytes_1 = Bytes::from("Unit test 1");
        let tag_name_1 = "Tag 1";
        let _ = add_blob_named(blobs.clone(), bytes_1, tag_name_1).await?;

        let tags = list_tags(blobs.clone()).await?;
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, tag_name_1.into());

        delete_tag(blobs.clone(), tag_name_1).await?;
        let tags_after_delete = list_tags(blobs).await?;
        assert_eq!(tags_after_delete.len(), 0);

        // Clean up
        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }    

    // export_blob_to_file
    #[tokio::test]
    pub async fn test_export_blob_to_file() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();
        let bytes = Bytes::from("Unit test");
        
        let outcome = add_blob_bytes(blobs.clone(), bytes).await?;
        let destination = std::fs::canonicalize(".")?.join("retrieved.txt");
        
        export_blob_to_file(blobs.clone(), outcome.hash.to_string(), destination.clone()).await?;

        // Check if the file exists and has the expected content
        let content = fs::read_to_string(destination).await?;
        assert_eq!(content, "Unit test");

        // Clean up
        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        fs::remove_file("retrieved.txt").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_export_blob_to_file_fails_on_invalid_hash() -> Result<()> {
        let iroh_node = setup_node().await?;
        let blobs = iroh_node.blobs.clone();
        let bytes = Bytes::from("Unit test");
        
        let _ = add_blob_bytes(blobs.clone(), bytes).await?;
        let destination = std::fs::canonicalize(".")?.join("retrieved.txt");

        let invalid_hash = "this is not a valid hash".to_string();
        
        let result = export_blob_to_file(blobs.clone(), invalid_hash, destination.clone()).await;

        assert!(matches!(result, Err(BlobError::InvalidBlobHashFormat)));

        // Clean up
        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }
}
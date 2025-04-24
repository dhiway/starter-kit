use iroh_blobs::{
    net_protocol::Blobs,
    rpc::client::blobs::WrapOption,
    store::mem::Store,
    // ticket::BlobTicket,
    util::SetTagOption,
    store::{ExportFormat, ExportMode},
    Hash,
};
use std::{path::{Path, PathBuf}, sync::Arc};
use anyhow::Result;

pub async fn save_file_to_blobs(
    blobs: Arc<Blobs<Store>>,
    file_path: &Path,
    in_place: bool,
) -> Result<Hash> {
    let blobs_client = blobs.client();

    let abs_path = std::fs::canonicalize(file_path)?;
    let blob = blobs_client
        .add_from_path(abs_path.clone(), in_place, SetTagOption::Auto, WrapOption::NoWrap)
        .await?
        .finish()
        .await?;

    // let node_id = blobs.endpoint().node_id();
    // let ticket = BlobTicket::new(node_id.into(), blob.hash, blob.format)?;
    let hash = blob.hash;

    println!("Saved file as blob with hash: {}", blob.hash);
    Ok(hash)
}

pub async fn export_blob_to_file(
    blobs: Arc<Blobs<Store>>,
    hash: Hash,
    destination: PathBuf,
) -> Result<()> {
    let blobs_client = blobs.client();

    blobs_client
        .export(hash, destination.clone() , ExportFormat::Blob, ExportMode::Copy)
        .await?
        .finish()
        .await?;
    println!("Exported blob with hash: {} to file: {:?}", hash, destination);

    Ok(())
}
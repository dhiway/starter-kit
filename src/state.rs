use std::sync::Arc;
use iroh_blobs::net_protocol::Blobs;
use iroh_docs::protocol::Docs;
use iroh_blobs::store::mem::Store as BlobStore;

#[derive(Clone)]
pub struct AppState {
    pub docs: Arc<Docs<BlobStore>>,
    pub blobs: Arc<Blobs<BlobStore>>,
}
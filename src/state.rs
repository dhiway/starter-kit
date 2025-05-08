use std::sync::Arc;
use iroh_blobs::net_protocol::Blobs;
use iroh_docs::protocol::Docs;
use iroh_blobs::store::fs::Store;

#[derive(Clone)]
pub struct AppState {
    pub docs: Arc<Docs<Store>>,
    pub blobs: Arc<Blobs<Store>>,
}
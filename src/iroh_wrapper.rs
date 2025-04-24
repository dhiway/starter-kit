use iroh::{Endpoint, RelayMode, SecretKey, protocol::Router};
use std::error::Error;
use iroh_blobs::{net_protocol::Blobs, store::mem::Store as blob_store};
use iroh_gossip::net::Gossip;
use iroh_docs::protocol::Docs;
use std::sync::Arc;

pub struct IrohNode {
    pub endpoint: Endpoint,
    pub node_id: String,
    pub router: Router,
    pub blobs: Arc<Blobs<blob_store>>,
    pub docs: Arc<Docs<blob_store>>,
}

/// Spins up an Iroh node with a default relay and returns the Endpoint and NodeId.
pub async fn setup_iroh_node() -> Result<IrohNode, Box<dyn Error>> {
    // For now, generate a new secret key each time (TODO: persist this later)
    let mut rng = rand::rngs::OsRng;
    let secret_key = SecretKey::generate(&mut rng);

    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .relay_mode(RelayMode::Default)
        .discovery_n0()
        .bind()
        .await?;

    let builder = Router::builder(endpoint.clone());

    let node_id = endpoint.clone().node_id().to_string();

    let blobs = Blobs::memory().build(builder.endpoint());
    let gossip = Gossip::builder().spawn(builder.endpoint().clone()).await?;
    let docs = Docs::memory().spawn(&blobs, &gossip).await?;
    
    let router = Router::builder(endpoint.clone())
        .accept(iroh_blobs::ALPN, blobs.clone())
        .accept(iroh_gossip::ALPN, gossip)
        .accept(iroh_docs::ALPN, docs.clone())
        .spawn()
        .await?;

    Ok(IrohNode {
        endpoint,
        node_id,
        router,
        blobs: Arc::new(blobs),
        docs: Arc::new(docs),
    })
}
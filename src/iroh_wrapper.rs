use iroh::{Endpoint, RelayMode, SecretKey, protocol::Router};
use std::error::Error;
use iroh_blobs::{net_protocol::Blobs, store::mem::Store as blob_store};
use iroh_gossip::net::Gossip;
use iroh_docs::protocol::Docs;
use std::sync::Arc;
use iroh::PublicKey;
use crate::cli::CliArgs;

pub struct IrohNode {
    pub node_id: PublicKey,
    pub router: Router,
    pub blobs: Arc<Blobs<blob_store>>,
    pub docs: Arc<Docs<blob_store>>,
}

pub async fn setup_iroh_node(args: CliArgs) -> Result<IrohNode, Box<dyn Error>> {
    // Use provided secret key if given, else generate new one
    let secret_key = if let Some(sk_hex) = args.secret_key {
        let bytes = hex::decode(&sk_hex)?;
        let bytes: [u8; 32] = bytes.try_into().expect("Invalid secret key length");
        SecretKey::from_bytes(&bytes)
    } else {
        let mut rng = rand::rngs::OsRng;
        SecretKey::generate(&mut rng)
    };
    println!("Secret key: {}", secret_key);

    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .relay_mode(RelayMode::Default)
        .discovery_n0()
        .bind()
        .await?;

    let builder = Router::builder(endpoint.clone());

    let node_id = endpoint.clone().node_id();

    let blobs = Blobs::memory().build(builder.endpoint());
    let gossip = Gossip::builder().spawn(builder.endpoint().clone()).await?;
    let docs = if let Some(path) = args.path {
        Docs::persistent(path).spawn(&blobs, &gossip).await?
    } else {
        Docs::memory().spawn(&blobs, &gossip).await?
    };
    
    let router = Router::builder(endpoint.clone())
        .accept(iroh_blobs::ALPN, blobs.clone())
        .accept(iroh_gossip::ALPN, gossip)
        .accept(iroh_docs::ALPN, docs.clone())
        .spawn()
        .await?;

    Ok(IrohNode {
        node_id,
        router,
        blobs: Arc::new(blobs),
        docs: Arc::new(docs),
    })
}
use iroh::{Endpoint, RelayMode, SecretKey, protocol::Router};
use std::error::Error;
use iroh_blobs::net_protocol::Blobs;
use iroh_blobs::store::fs::Store as blob_store_fs;
use iroh_gossip::net::Gossip;
use iroh_docs::protocol::Docs;
use std::sync::Arc;
use iroh::PublicKey;
use crate::cli::CliArgs;

pub struct IrohNode {
    pub node_id: PublicKey,
    pub router: Router,
    pub blobs: Arc<Blobs<blob_store_fs>>,
    pub docs: Arc<Docs<blob_store_fs>>,
}

pub async fn setup_iroh_node(args: CliArgs) -> Result<IrohNode, Box<dyn Error>> {
    let path = args.path.expect("Path is required");

    let bytes = match args.secret_key {
        Some(ref secret_key) => hex::decode(secret_key)?,
        None => return Err("Secret key is required".into()),
    };
    let bytes: [u8; 32] = bytes.try_into().expect("Invalid secret key length");
    let secret_key = SecretKey::from_bytes(&bytes);
    println!("Secret key: {}", secret_key);

    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .relay_mode(RelayMode::Default)
        .discovery_n0()
        .bind()
        .await?;

    let builder = Router::builder(endpoint.clone());

    let node_id = endpoint.clone().node_id();

    let blobs = Blobs::persistent(path.clone()).await?.build(builder.endpoint());
    let gossip = Gossip::builder().spawn(builder.endpoint().clone()).await?;
    let docs = Docs::persistent(path).spawn(&blobs, &gossip).await?;
    
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
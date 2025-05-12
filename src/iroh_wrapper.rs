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
    // The user is first expected to create a secret key using ```cago run``` and then passing it as an argument
    let (path, secret_key) = match (&args.path, &args.secret_key) {
        (Some(path), Some(secret_key)) => {
            let bytes = hex::decode(secret_key)
                .map_err(|_| "Invalid hex format for secret key")?;

            let bytes: [u8; 32] = bytes
                .try_into()
                .map_err(|_| "Invalid secret key length, must be 64 hex chars")?;

            (path.clone(), SecretKey::from_bytes(&bytes))
        }
        (Some(_), None) => {
            return Err("Secret key is required when path is provided".into());
        }
        (None, Some(_)) => {
            return Err("Path is required when secret key is provided".into());
        }
        (None, None) => {
            let mut rng = rand::rngs::OsRng;
            let secret_key = SecretKey::generate(&mut rng);

            println!("Secret key: {}", secret_key);

            println!("🔑 Generated new secret key: {secret_key}");
            println!("👉 Please run again with:");
            println!("   cargo run -- --path your-path-of-choice --secret-key {secret_key}");

            return Err("Path and secret key are required".into());
        }
    };

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
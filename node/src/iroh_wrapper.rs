use helpers::cli::CliArgs;
// use keystore::StarterkitKeystore;
use keystore::keystore::{StarterkitKeystore, CordKeystoreSigner};

use iroh::{Endpoint, RelayMode, SecretKey, protocol::Router};
use std::error::Error;
use std::path::PathBuf;
use std::fs;
use tokio::fs as tokio_fs;
use tokio::io::AsyncWriteExt;
use iroh_blobs::net_protocol::Blobs;
use iroh_blobs::store::fs::Store as blob_store_fs;
use iroh_gossip::net::Gossip;
use iroh_docs::protocol::Docs;
use std::sync::Arc;
use iroh::PublicKey;

pub struct IrohNode {
    pub node_id: PublicKey,
    pub router: Router,
    pub blobs: Arc<Blobs<blob_store_fs>>,
    pub docs: Arc<Docs<blob_store_fs>>,
    pub cord_signer: CordKeystoreSigner,
}

pub async fn setup_iroh_node(args: CliArgs) -> Result<IrohNode, Box<dyn Error>> {
    // password should always be provided
    if args.password.is_empty() {
        return Err("❌ Password is required. Please provide --password <PASSWORD>.".into());
    }

    // if bootstrap is not set, then suri and secret key should not be set
    if !args.bootstrap {
        if args.suri.is_some() {
            return Err("❌ SURI (--suri) can only be provided when bootstrapping a new node (--bootstrap).".into());
        }

        // a logic for handling presence of --secret in absence of --bootstrap is not present as the 
        // user could pass the --secret and it will still work.
    }

    let mut path: PathBuf;
    let mut secret_key: SecretKey;
    let mut cord_signer: CordKeystoreSigner;

    // it is a bootstrap operation or a restart operation
    if args.bootstrap {
        println!("🚀 Bootstrapping process begun...\n");

        // bootstrap logic here
        if args.suri.is_none() {
            return Err("❌ SURI is required for bootstrapping. Please provide --suri <SURI>.".into());
        }

        path = args.path
            .as_ref()
            .map(|p| PathBuf::from(p))
            .unwrap_or_else(|| PathBuf::from("data"));
        let password = args.password.clone();
        let suri = args.suri.unwrap().clone();
        let secret = args.secret.clone();

        // if a node is already configured at the path, then throw an error
        if path.exists() {
            return Err(format!(
                "❌ A node is already configured at {:?}.\n\
                Please choose a different path with --path <PATH> or remove the existing node directory.",
                path
            ).into());
        }

        fs::create_dir_all(&path).map_err(|e| format!("Failed to create directory: {}", e))?;
        println!("✅ Created directory at {:?}\n", path);
        
        // create a file inside path called 'password' and store the hash of the password in it.
        let mut password_file_path = path.clone();
        password_file_path.push("password");
        let password_hash = blake3::hash(password.as_bytes()).to_hex().to_string();
        fs::write(&password_file_path, password_hash)
            .map_err(|e| format!("❌ Failed to create directory {:?}: {e}", path))?;
        println!("✅ Password successfully set for node.\n");

        // create a directory inside path called 'keystore' 
        let mut keystore_dir = path.clone();
        keystore_dir.push("keystore");
        fs::create_dir_all(&keystore_dir)
            .map_err(|e| format!("❌ Failed to write password file at {:?}: {e}", password_file_path))?;

        // setup the keystore
        let keystore_secret = StarterkitKeystore::keystore_access(secret)
            .map_err(|e| format!("❌ Failed to process keystore secret: {e}"))?;

        let mut keystore = StarterkitKeystore::new(&keystore_dir, keystore_secret)
            .map_err(|e| format!("❌ Failed to initialize keystore: {e}"))?;

        let (cord_pair, starter_kit_pair) = keystore
            .initialize_keystore(&suri.clone())
            .map_err(|e| format!("❌ Failed to initialize keypairs in keystore: {e}"))?;

        secret_key = keystore
            .get_starter_kit_seed(starter_kit_pair)
            .map_err(|e| format!("❌ Failed to get starter kit seed: {e}"))?;
        
        println!("✅ Keystore initialized successfully.\n");

        cord_signer = keystore.get_cord_signer()?;

        println!("🎉 Bootstarpping process completed successfully.\n");
    } else {
        println!("🔄 Restarting process begun...\n");

        path = args.path
            .as_ref()
            .map(|p| PathBuf::from(p))
            .unwrap_or_else(|| PathBuf::from("data"));

        if !path.exists() {
            return Err(format!(
                "❌ The provided path {:?} does not exist.\n\
                Please bootstrap a new node first using --bootstrap.",
                path
            ).into());
        }

        let password = args.password.clone();

        println!("🔑 Checking password for the node at {:?}\n", path);
        let mut password_file_path = path.clone();
        password_file_path.push("password");

        let password_hash = blake3::hash(password.as_bytes()).to_hex().to_string();

        let stored_password_hash = fs::read_to_string(&password_file_path)
            .map_err(|e| format!("❌ Failed to read password file: {}", e))?;

        if password_hash != stored_password_hash {
            return Err("❌ Incorrect password provided. Please check your --password and try again.".into());
        }
        println!("✅ Password verified successfully.\n");

        let mut keystore_dir = path.clone();
        keystore_dir.push("keystore");

        println!("🔐 Opening keystore at {:?}\n", keystore_dir);

        // NOTE: thought that to restart a node, we would need to take in 'secret', not because 
        //       it is like a password that is required everytime to open the keystore but as it would
        //       required to fetch the secret key to start the iroh node. But that is not the case.
        //       It is still in place, in case someone passes --secret or not, it will still work.
        //       PROOF:
        //       After bootsrapping a node with 'secret' argument, if the user tries to restart the 
        //       node with/without the 'secret' argument, the 'secret_key' variable will be the same.
        let keystore_secret = StarterkitKeystore::keystore_access(args.secret.clone())
            .map_err(|e| format!("❌ Failed to process keystore secret: {e}"))?;

        let keystore = StarterkitKeystore::open(&keystore_dir, keystore_secret)
            .map_err(|e| format!("❌ Failed to open keystore: {e}"))?;

        let starterkit_public = keystore
            .get_starterkit_public_key()
            .map_err(|e| format!("❌ Failed to get starterkit public key: {e}"))?;

        secret_key = keystore
            .get_starter_kit_seed(starterkit_public)
            .map_err(|e| format!("❌ Failed to get starterkit seed: {e}"))?;

        println!("✅ Keystore opened successfully.\n");

        cord_signer = keystore.get_cord_signer()?;

        println!("🎉 Restarting process completed successfully.\n");
    }

    let endpoint = Endpoint::builder()
        .secret_key(secret_key.clone())
        .relay_mode(RelayMode::Default)
        .discovery_n0()
        .bind()
        .await?;

    let builder = Router::builder(endpoint.clone());

    let node_id = endpoint.clone().node_id();

    let blobs = Blobs::persistent(path.clone()).await?.build(builder.endpoint());
    let gossip = Gossip::builder().spawn(builder.endpoint().clone()).await?;
    let docs = Docs::persistent(path.clone()).spawn(&blobs, &gossip).await?;
    
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
        cord_signer,
    })
}
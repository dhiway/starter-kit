use iroh::{Endpoint, RelayMode, SecretKey};
use std::error::Error;

/// Spins up an Iroh node with a default relay and returns the Endpoint and NodeId.
pub async fn setup_iroh_node() -> Result<(Endpoint, String), Box<dyn Error>> {
    // For now, generate a new secret key each time (TODO: persist this later)
    let mut rng = rand::rngs::OsRng;
    let secret_key = SecretKey::generate(&mut rng);

    // Use default libp2p configuration (includes default relay + discovery)
    // let config = Libp2pConfig::default();

    // Create a new endpoint with key and config
    let builder = Endpoint::builder()
        .secret_key(secret_key.clone())
        .relay_mode(RelayMode::Default)
        .discovery_n0();

    let endpoint = builder.bind().await?;

    let node_id = endpoint.node_id().to_string();

    Ok((endpoint, node_id))
}
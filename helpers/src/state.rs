use keystore::keystore::CordKeystoreSigner;

use std::sync::Arc;
use iroh_blobs::net_protocol::Blobs;
use iroh_docs::protocol::Docs;
use iroh_blobs::store::fs::Store;
use subxt_rpcs::RpcClient;
use subxt::client::OnlineClient;
use subxt::config::PolkadotConfig;

#[derive(Clone)]
pub struct AppState {
    pub docs: Arc<Docs<Store>>,
    pub blobs: Arc<Blobs<Store>>,
    // pub cord_client: Arc<RpcClient>,
    pub cord_client: Arc<OnlineClient<PolkadotConfig>>,
    pub cord_signer: CordKeystoreSigner
}
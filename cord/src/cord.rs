use subxt_rpcs::{RpcClient, ChainHeadRpcMethods, rpc_params};
use subxt::config::PolkadotConfig; // or your chain's config
use subxt::client::OnlineClient;

pub async fn connect_to_chain() -> Result<OnlineClient<PolkadotConfig>, Box<dyn std::error::Error>> {
    println!("⛓ Connecting to chain...");

    // let client = RpcClient::from_url("ws://127.0.0.1:9944").await?;
    let client = OnlineClient::<PolkadotConfig>::from_url("ws://127.0.0.1:9944").await?;

    // let chain: String = client.request("system_chain", rpc_params![]).await?;
    // println!("✅ Successfully connected to chain: {}", chain);

    println!("⛓ Connected to chain");

    Ok(client)
}
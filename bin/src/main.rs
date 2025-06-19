use node::iroh_wrapper::{setup_iroh_node, IrohNode};
use router::router::create_router;
use helpers::{
    cli::CliArgs,
    frontend::start_frontend,
    state::AppState,
};
use gateway::{
    storage::init_access_control,
    access_control::{set_storage_path, ensure_self_node_id_allowed},
};

use tokio::signal;
use std::error::Error;
use clap::Parser;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse CLI arguments
    let args = CliArgs::parse();

    // Initialize the Iroh node
    let iroh_node: IrohNode = setup_iroh_node(args.clone()).await?;

    // Initialize gateway
    let path = args.path.unwrap();
    let path_str = path.to_str().unwrap();
    let (mut allowed_node_ids, allowed_domains) = init_access_control(path_str).await?;

    // Ensure self NodeId is added on first run
    ensure_self_node_id_allowed(
        &path_str.to_string().clone(), 
        iroh_node.node_id.to_string().clone(), 
        &mut allowed_node_ids
    ).await?;

    set_storage_path(
        path_str.to_string(), 
        allowed_node_ids, 
        allowed_domains
    );

    // Start frontend
    // start_frontend();

    println!("Iroh node started!");
    println!("Your NodeId: {}", iroh_node.node_id);

    let state = AppState {
        blobs: iroh_node.blobs.clone(),
        docs: iroh_node.docs.clone(),
    };

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4001").await?;
    println!("Server started on http://localhost:4001");

    axum::serve(listener, app).await?;
    
    println!("Press Ctrl+C to shut down...");

    // Wait for Ctrl+C signal
    signal::ctrl_c().await?;
    println!("\nShutdown signal received. Exiting...");
    iroh_node.router.shutdown().await?;

    Ok(())
}
use dept_starter_kit_template::{
    node::iroh_wrapper::{setup_iroh_node, IrohNode},
    router::router::create_router,
    helpers::{
        cli::CliArgs,
        frontend::start_frontend,
        state::AppState,
    }
};
use tokio::signal;
use std::error::Error;
use clap::Parser;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse CLI arguments
    let args = CliArgs::parse();

    // Initialize the Iroh node
    let iroh_node: IrohNode = setup_iroh_node(args).await?;

    // Start frontend
    start_frontend();

    println!("Iroh node started!");
    println!("Your NodeId: {}", iroh_node.node_id);

    let state = AppState {
        blobs: iroh_node.blobs.clone(),
        docs: iroh_node.docs.clone(),
    };

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4000").await?;
    println!("Server started on http://localhost:4000");

    axum::serve(listener, app).await?;
    
    println!("Press Ctrl+C to shut down...");

    // Wait for Ctrl+C signal
    signal::ctrl_c().await?;
    println!("\nShutdown signal received. Exiting...");
    iroh_node.router.shutdown().await?;

    Ok(())
}
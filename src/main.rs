mod iroh_wrapper;
use crate::iroh_wrapper::setup_iroh_node;
use tokio::signal;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the Iroh node
    let (_node, node_id) = setup_iroh_node().await?;

    println!("Iroh node started!");
    println!("Your NodeId: {}", node_id);
    println!("Press Ctrl+C to shut down...");

    // Wait for Ctrl+C signal
    signal::ctrl_c().await?;
    println!("\nShutdown signal received. Exiting...");

    // Node shutdown logic can go here if needed
    Ok(())
}
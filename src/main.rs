mod iroh_wrapper;
mod blobs;
mod docs;
mod helper;
mod handlers;
mod state;

use iroh_wrapper::{setup_iroh_node, IrohNode};
use tokio::signal;
use std::error::Error;
use std::process::Command;
use axum::{Router, routing::{post, get}};
use tower_http::cors::CorsLayer;
use handlers::{create_registry_handler, get_all_registries_handler, archive_registry_handler, add_entry_handler, display_entry_handler, delete_entry_handler};
use state::AppState;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Start frontend
    let frontend = Command::new("npm")
        .arg("start")
        .current_dir("frontend")
        .spawn();

    match frontend {
        Ok(_) => println!("✅ Frontend server started on http://localhost:3000"),
        Err(e) => eprintln!("❌ Failed to start frontend server: {}", e),
    }

    // Initialize the Iroh node
    let iroh_node: IrohNode = setup_iroh_node().await?;

    println!("Iroh node started!");
    println!("Your NodeId: {}", iroh_node.node_id);

    let state = AppState {
        docs: iroh_node.docs.clone(),
        blobs: iroh_node.blobs.clone(),
    };

    let app = Router::new()
        .route("/create_registry", post(create_registry_handler))
        .route("/all_registries", get(get_all_registries_handler))
        .route("/archive", post(archive_registry_handler))
        .route("/add_entry", post(add_entry_handler))
        .route("/display_entries", post(display_entry_handler))
        .route("/delete_entry", post(delete_entry_handler))
        .with_state(state)
        .layer(CorsLayer::very_permissive());

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
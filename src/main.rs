use dept_starter_kit_template::node::iroh_wrapper::{setup_iroh_node, IrohNode};
use dept_starter_kit_template::helpers::state::AppState;
use dept_starter_kit_template::helpers::cli::CliArgs;
use dept_starter_kit_template::API_handlers::{
    blobs_handler::{add_blob_bytes_handler, add_blob_named_handler, list_blobs_handler, add_blob_from_path_handler, get_blob_handler, status_blob_handler, has_blob_handler, download_blob_handler, download_hash_sequence_handler, download_with_options_handler, list_tags_handler, delete_tag_handler, export_blob_to_file_handler},
    authors_handler::{list_authors_handler, get_default_author_handler, set_default_author_handler, create_author_handler, delete_author_handler, verify_author_handler},
    docs_handler::{get_document_handler, get_entry_blob_handler, create_doc_handler, list_docs_handler, drop_doc_handler, share_doc_handler, join_doc_handler, close_doc_handler, add_doc_schema_handler, set_entry_handler, set_entry_file_handler, get_entry_handler, get_entries_handler, delete_entry_handler, leave_handler, status_handler, set_download_policy_handler, get_download_policy_handler},
};
use dept_starter_kit_template::router::create_router;
use dept_starter_kit_template::helper::frontend::start_frontend;
use tokio::signal;
use std::error::Error;
use std::process::Command;
use axum::{routing::{get, post}, Router};
use tower_http::cors::CorsLayer;
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
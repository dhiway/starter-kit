use dept_starter_kit_template::node::iroh_wrapper::{setup_iroh_node, IrohNode};
use dept_starter_kit_template::helpers::state::AppState;
use dept_starter_kit_template::helpers::cli::CliArgs;
use dept_starter_kit_template::API_handlers::{
    blobs_handler::{add_blob_bytes_handler, add_blob_named_handler, list_blobs_handler, add_blob_from_path_handler, get_blob_handler, status_blob_handler, has_blob_handler, download_blob_handler, download_hash_sequence_handler, download_with_options_handler, list_tags_handler, delete_tag_handler, export_blob_to_file_handler},
    authors_handler::{list_authors_handler, get_default_author_handler, set_default_author_handler, create_author_handler, delete_author_handler, verify_author_handler},
};
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

    // // Start frontend
    // let frontend = Command::new("npm")
    //     .arg("start")
    //     .current_dir("frontend")
    //     .spawn();

    // match frontend {
    //     Ok(_) => println!("✅ Frontend server started on http://localhost:3000"),
    //     Err(e) => eprintln!("❌ Failed to start frontend server: {}", e),
    // }

    println!("Iroh node started!");
    println!("Your NodeId: {}", iroh_node.node_id);

    let state = AppState {
        blobs: iroh_node.blobs.clone(),
        docs: iroh_node.docs.clone(),
    };

    let app = Router::new()
        .route("/blobs/add-blob-bytes", post(add_blob_bytes_handler))
        .route("/blobs/add-blob-named", post(add_blob_named_handler))
        .route("/blobs/add-blob-from-path", post(add_blob_from_path_handler))
        .route("/blobs/list-blobs", get(list_blobs_handler))
        .route("/blobs/get-blob", get(get_blob_handler))
        .route("/blobs/status-blob", get(status_blob_handler))
        .route("/blobs/has-blob", get(has_blob_handler))
        .route("/blobs/download-blob", get(download_blob_handler))
        .route("/blobs/download-hash-sequence", get(download_hash_sequence_handler))
        .route("/blobs/download-with-options", get(download_with_options_handler))
        .route("/blobs/list-tags", get(list_tags_handler))
        .route("/blobs/delete-tag", post(delete_tag_handler))
        .route("/blobs/export-blob-to-file", post(export_blob_to_file_handler))
        .route("/authors/list-authors", get(list_authors_handler))
        .route("/authors/get-default-author", get(get_default_author_handler))
        .route("/authors/set-default-author", post(set_default_author_handler))
        .route("/authors/create-author", post(create_author_handler))
        .route("/authors/delete-author", post(delete_author_handler))
        .route("/authors/verify-author", post(verify_author_handler))
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
mod iroh_wrapper;
mod blobs;
mod docs;
mod helper;
mod handlers;
mod state;
mod utils;

use iroh_wrapper::{setup_iroh_node, IrohNode};
use tokio::signal;
use core::time;
use std::error::Error;
use std::process::Command;
use axum::{Router, routing::{post, get}};
use tower_http::cors::CorsLayer;
use handlers::{create_registry_handler, get_all_registries_handler, archive_registry_handler, add_entry_handler, display_entry_handler, delete_entry_handler};
use state::AppState;
use docs::{get_document, save_as_doc, create_doc, list_docs, add_doc_schema, fetch_doc_as_json, set_entry};

use std::collections::BTreeMap;
use serde_json::Value;
use tokio::time::{sleep, Duration};


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

    // let docId = create_doc(state.docs.clone()).await?;
    // println!("Doc saved with id: {:?}", docId);

    // sleep(Duration::from_secs(5)).await;

    // let mut schema: BTreeMap<String, Value> = BTreeMap::new();
    // schema.insert("name".to_string(), Value::String("string".to_string()));
    // schema.insert("age".to_string(), Value::String("integer".to_string()));
    // schema.insert("is_member".to_string(), Value::String("boolean".to_string()));
    // println!("JSON: {:#?}", schema);

    // let hash = add_doc_schema(state.docs.clone(), docId. clone(), schema.clone()).await?;
    // println!("Schema added with hash: {:?}", hash);

    // sleep(Duration::from_secs(5)).await;

    // let keys: Vec<&str> = vec!["schema"];
    // let schema = fetch_doc_as_json(state.docs.clone(), state.blobs.clone(), docId.clone(), Some(keys)).await?;
    // println!("Document fetched: {:#?}", schema);

    // sleep(Duration::from_secs(5)).await;

    // let mut entry: BTreeMap<String, Value> = BTreeMap::new();
    // entry.insert("name".to_string(), Value::String("Alice".to_string()));
    // entry.insert("age".to_string(), Value::Number(20.into()));
    // entry.insert("is_member".to_string(), Value::Bool(true));
    // println!("JSON: {:#?}", entry);

    // let entry_hash = set_entry(state.docs.clone(), state.blobs.clone(), docId.clone(), "name".to_string(), Value::String("Alice".to_string())).await?;
    // println!("Entry added with hash: {:?}", entry_hash);
    // sleep(Duration::from_secs(2)).await;
    // let entry_hash = set_entry(state.docs.clone(), state.blobs.clone(), docId.clone(), "age".to_string(), Value::Number(20.into())).await?;
    // println!("Entry added with hash: {:?}", entry_hash);
    // sleep(Duration::from_secs(2)).await;
    // let entry_hash = set_entry(state.docs.clone(), state.blobs.clone(), docId.clone(), "is_member".to_string(), Value::Number(20.into())).await?;
    // println!("Entry added with hash: {:?}", entry_hash);
    // sleep(Duration::from_secs(2)).await;

    // let keys: Vec<&str> = vec!["schema", "name", "age", "is_member"];
    // let schema = fetch_doc_as_json(state.docs.clone(), state.blobs.clone(), docId.clone(), Some(keys)).await?;
    // println!("Document fetched: {:#?}", schema);

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
mod iroh_wrapper;
mod blobs;
mod docs;
mod helper;

use crate::iroh_wrapper::{setup_iroh_node, IrohNode};
use crate::blobs::{save_file_to_blobs, export_blob_to_file};
use crate::docs::save_as_doc;
use crate::docs::fetch_doc_as_json;
use tokio::signal;
use std::error::Error;
use serde_json::json;
use std::{collections::BTreeMap, path::{Path, PathBuf}, thread, time};
use std::process::Command;
use crate::helper::{create_registry, show_all_registry, archive_registry};
use iroh_docs::NamespaceId;
use axum::{Router, routing::post, Extension};
use std::sync::Arc;
use iroh_blobs::net_protocol::Blobs;
use iroh_docs::protocol::Docs;
use iroh_blobs::store::mem::Store as BlobStore;
use tower_http::cors::{CorsLayer, Any};

#[derive(Clone)]
pub struct AppState {
    pub docs: Arc<Docs<BlobStore>>,
    pub blobs: Arc<Blobs<BlobStore>>,
}

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
    
    // // Try saving a file
    // let hash = save_file_to_blobs(iroh_node.blobs.clone(), Path::new("doctors_1998.csv"), true).await?;
    // // println!("Blob ticket: {}", ticket);

    // let five_sec = time::Duration::from_millis(5000);
    // thread::sleep(five_sec);

    // // Try exporting a blob to a file
    // let destination = std::fs::canonicalize(".")?.join("retrieved.txt");
    // export_blob_to_file(iroh_node.blobs.clone(), hash, destination).await?;
    
    // // Try saving a doc
    // let mut doc_data = BTreeMap::new();
    // doc_data.insert("owner".to_string(), json!("dhiway"));
    // doc_data.insert("version".to_string(), json!(1));
    // doc_data.insert("hash".to_string(), json!(hash));
    // doc_data.insert("entry_count".to_string(), json!(5));

    // let doc_id = save_as_doc(iroh_node.docs.clone(), doc_data).await?;
    // println!("Doc saved with id: {}", doc_id);

    // thread::sleep(five_sec);

    // fetch_doc_as_json(iroh_node.docs.clone(), iroh_node.blobs.clone(), doc_id).await?;

    // let schema = r#"{"owner": "string", "version": "int", "entry_count": "int", "namespace_id": "str"}"#;
    // let file_path = "doctors_1998.csv";
    // let name = "doctors_1998";
    // match create_registry(iroh_node.blobs.clone(), iroh_node.docs.clone(), name, schema, file_path).await {
    //     Ok(doc_key) => {
    //         println!("✅ Registry created with key: {}", doc_key);
    //         thread::sleep(time::Duration::from_millis(5000));
    //         fetch_doc_as_json(iroh_node.docs.clone(), iroh_node.blobs.clone(), doc_key).await?;
    //     },
    //     Err(e) => eprintln!("❌ Error: {}", e),
    // }

    // let schema = r#"{"owner": "string", "version": "int", "entry_count": "int", "namespace_id": "str"}"#;
    // let file_path = "doctors_2020.csv";
    // let name = "doctors_2020";
    // match create_registry(iroh_node.blobs.clone(), iroh_node.docs.clone(), name, schema, file_path).await {
    //     Ok(doc_key) => {
    //         println!("✅ Registry created with key: {}", doc_key);

    //         thread::sleep(time::Duration::from_millis(5000));
    //         fetch_doc_as_json(iroh_node.docs.clone(), iroh_node.blobs.clone(), doc_key).await?;

    //         thread::sleep(time::Duration::from_millis(5000));
    //         archive_registry(iroh_node.docs.clone(), iroh_node.blobs.clone(), "doctors_2020").await?;
    //         println!("✅ Registry archived");

    //         thread::sleep(time::Duration::from_millis(5000));
    //         fetch_doc_as_json(iroh_node.docs.clone(), iroh_node.blobs.clone(), doc_key).await?;

    //         // thread::sleep(time::Duration::from_millis(5000));
    //         // archive_registry(iroh_node.docs.clone(), iroh_node.blobs.clone(), "doctors_2020").await?;
    //     },
    //     Err(e) => println!("❌ Error: {}", e),
    // }

    // thread::sleep(time::Duration::from_millis(5000));
    // show_all_registry();

    
    println!("Press Ctrl+C to shut down...");

    // Wait for Ctrl+C signal
    signal::ctrl_c().await?;
    println!("\nShutdown signal received. Exiting...");
    iroh_node.router.shutdown().await?;

    Ok(())
}
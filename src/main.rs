mod iroh_wrapper;
mod blobs;
mod docs;

use crate::iroh_wrapper::{setup_iroh_node, IrohNode};
use crate::blobs::{save_file_to_blobs, export_blob_to_file};
use crate::docs::save_as_doc;
use crate::docs::fetch_doc_as_json;
use tokio::signal;
use std::error::Error;
use serde_json::json;
use std::{collections::BTreeMap, path::{Path, PathBuf}, thread, time};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the Iroh node
    let irohNode: IrohNode = setup_iroh_node().await?;

    println!("Iroh node started!");
    println!("Your NodeId: {}", irohNode.node_id);
    
    // Try saving a file
    let hash = save_file_to_blobs(irohNode.blobs.clone(), Path::new("doctors_1998.csv"), true).await?;
    // println!("Blob ticket: {}", ticket);

    let five_sec = time::Duration::from_millis(5000);
    thread::sleep(five_sec);

    // Try exporting a blob to a file
    let destination = std::fs::canonicalize(".")?.join("retrieved.txt");
    export_blob_to_file(irohNode.blobs.clone(), hash, destination).await?;
    
    // Try saving a doc
    let mut doc_data = BTreeMap::new();
    doc_data.insert("owner".to_string(), json!("dhiway"));
    doc_data.insert("version".to_string(), json!(1));
    doc_data.insert("hash".to_string(), json!(hash));
    doc_data.insert("entry_count".to_string(), json!(5));

    let doc_id = save_as_doc(irohNode.docs.clone(), doc_data).await?;
    println!("Doc saved with id: {}", doc_id);

    thread::sleep(five_sec);

    fetch_doc_as_json(irohNode.docs.clone(), irohNode.blobs.clone(), doc_id).await?;
    
    println!("Press Ctrl+C to shut down...");

    // Wait for Ctrl+C signal
    signal::ctrl_c().await?;
    println!("\nShutdown signal received. Exiting...");
    irohNode.router.shutdown().await?;

    Ok(())
}
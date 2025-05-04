mod iroh_wrapper;
mod blobs;
mod docs;
mod helper;
mod handlers;
mod state;
mod utils;
mod authors;
mod cli;

use iroh_wrapper::{setup_iroh_node, IrohNode};
use tokio::signal;
use core::time;
use std::error::Error;
use std::process::Command;
use axum::{extract::Path as AxumPath, routing::{get, post}, Router};
use std::path::Path;
use tower_http::cors::CorsLayer;
use handlers::{create_registry_handler, get_all_registries_handler, archive_registry_handler, add_entry_handler, display_entry_handler, delete_entry_handler};
use state::AppState;
use docs::{add_doc_schema, close_doc, create_doc, delete_entry, drop_doc, fetch_doc_as_json, get_document, get_download_policy, get_entries, get_entry, get_entry_blob, join_doc, leave, list_docs, save_as_doc, set_download_policy, set_entry, set_entry_file, share_doc, status, ImportFileOutcome};
use utils::{decode_doc_id, SS58AuthorId};
use blobs::{add_blob_bytes, add_blob_named, add_blob_from_path, list_blobs, has_blob, get_blob, status_blob, list_tags, delete_tag, export_blob_to_file, download_blob, download_hash_sequence, download_with_options};
use authors::{list_authors, get_default_author, set_default_author, create_author, delete_author, verify_author};
use anyhow::Context;
use iroh_docs::NamespaceId;
use iroh_docs::rpc::AddrInfoOptions;
use iroh_docs::rpc::client::docs::ShareMode;
use iroh_blobs::rpc::client::blobs::{AddOutcome, DownloadOutcome, DownloadOptions};
use std::path::PathBuf;
use iroh_blobs::util::SetTagOption;
use crate::cli::CliArgs;
use clap::Parser;

use std::collections::BTreeMap;
use serde_json::{Value, json};
use tokio::time::{sleep, Duration};


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse CLI arguments
    let args = CliArgs::parse();
    println!("Args: {:#?}", args);

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
    let iroh_node: IrohNode = setup_iroh_node(args).await?;

    println!("Iroh node started!");
    println!("Your NodeId: {}", iroh_node.node_id);
    println!("Your NodeId as a byte array is: {:?}", iroh_node.node_id.as_bytes());

    let state = AppState {
        docs: iroh_node.docs.clone(),
        blobs: iroh_node.blobs.clone(),
    };

    // // // Test scenario 1: Basic Document Lifecycle(covers create_doc, add_doc_schema, set_entry, get_entry, get_entries, delete_entry)

    // let doc_id = create_doc(state.docs.clone()).await?;
    // println!("Document created with ID: {:?}", doc_id);
    
    // sleep(Duration::from_secs(3)).await;

    // let doc_client = state.docs.client();
    // let author_id = doc_client.authors().default().await?;
    // let author_ss58 = SS58AuthorId::from_author_id(&author_id)?;
    // let author = author_ss58.as_ss58();
    // println!("Document author: {:?}", author);

    // let schema = r#"{
    //     "type": "object",
    //     "properties": {
    //       "owner": { "type": "string" },
    //       "name": { "type": "string" },
    //       "number_of_entries": { "type": "integer" },
    //       "terms_and_conditions": { "type": "string" }
    //     },
    //     "required": ["owner", "name", "number_of_entries", "terms_and_conditions"]
    //   }"#;
    // let schema_id = add_doc_schema(state.docs.clone(), author.to_string(), doc_id.clone(), schema.to_string()).await?;
    // println!("Schema added with ID: {:?}", schema_id);

    // sleep(Duration::from_secs(3)).await;

    // let entry_1 = json!({
    //     "owner": "Dhiway",
    //     "name": "Cyra",
    //     "number_of_entries": 3,
    //     "terms_and_conditions": "Agreed"
    // });
    // let entry = set_entry(
    //     state.docs.clone(),
    //     state.blobs.clone(),
    //     doc_id.clone(),
    //     author.to_string(),
    //     "entry_1".to_string(),
    //     entry_1.to_string()
    // ).await?;
    // println!("Entry added with ID: {:?}", entry);

    // let entry_2 = json!({
    //     "owner": "Dhiway",
    //     "name": "DeDi",
    //     "number_of_entries": 5,
    //     "terms_and_conditions": "Not Agreed"
    // });
    // let entry = set_entry(
    //     state.docs.clone(),
    //     state.blobs.clone(),
    //     doc_id.clone(),
    //     author.to_string(),
    //     "entry_2".to_string(),
    //     entry_2.to_string()
    // ).await?;
    // println!("Entry added with ID: {:?}", entry);

    // sleep(Duration::from_secs(3)).await;

    // if let Some(get_owner_entry) = get_entry(state.docs.clone(), doc_id.clone(), author.to_string(), "entry_1".to_string(), false).await? {
    //     println!("entry_1 entry fetched: {:?}", get_owner_entry);
    // } else {
    //     println!("entry_1 entry not found");
    // }

    // sleep(Duration::from_secs(3)).await;

    // // try fetching all the entries(expect 3: schema, entry_1, entry_2)
    // let all_entries = get_entries(
    //     state.docs.clone(), 
    //     doc_id.clone(), 
    //     json!({
    //         "author_id": author,
    //         "sort_by": "key",
    //         "sort_direction": "ascending"
    //     }),
    // ).await?;
    // println!("All entries for doc before deletion:");
    // for entry in all_entries {
    //     println!("{:?}", entry);
    // }

    // sleep(Duration::from_secs(3)).await;

    // let deletion_size = delete_entry(
    //     state.docs.clone(),
    //     doc_id.clone(),
    //     author.to_string(),
    //     "entry_1".to_string(),
    // ).await?;
    // println!("Deleted entry size: {:?}", deletion_size);

    // sleep(Duration::from_secs(3)).await;

    // // try fetching all the entries(expect 2: schema, entry_2)
    // let all_entries = get_entries(
    //     state.docs.clone(), 
    //     doc_id.clone(), 
    //     json!({
    //         "author_id": author,
    //         "sort_by": "key",
    //         "sort_direction": "ascending"
    //     }),
    // ).await?;
    // println!("All entries for doc after deletion:");
    // for entry in all_entries {
    //     println!("{:?}", entry);
    // }

    // Test scenario 2: Entry via File(covers set_entry_file, get_entry_blob)

    // let doc_id = create_doc(state.docs.clone()).await?;
    // println!("Document created with ID: {:?}", doc_id);

    // sleep(Duration::from_secs(3)).await;

    // let doc_client = state.docs.client();
    // let author_id = doc_client.authors().default().await?;
    // let author_ss58 = SS58AuthorId::from_author_id(&author_id)?;
    // let author = author_ss58.as_ss58();
    // println!("Document author: {:?}", author);

    // let import_file_outcome: ImportFileOutcome = set_entry_file(
    //     state.docs.clone(),
    //     doc_id.clone(),
    //     author.to_string(),
    //     "entry_1".to_string(),
    //     "_".to_string(), // add absolute path to the file to upload
    // ).await?;
    // println!("Entry added with ID: {:?}", import_file_outcome);

    // sleep(Duration::from_secs(3)).await;

    // let file_content = get_entry_blob(state.blobs.clone(), import_file_outcome.hash).await?;
    // println!("Blob content: {:?}", file_content);

    // Test scenario 3: Blob Store - Add and Tag. Blob Queries(covers add_blob_bytes, add_blob_named, add_blob_from_path, list_blobs, has_blob, get_blob, status_blob, list_tags, delete_tag, export_blob_to_file, download_with_options)

    // let add_blob_bytes_outcome = add_blob_bytes(
    //     state.blobs.clone(), 
    //     "Hello, world!".as_bytes()
    // ).await?;
    // println!("Bytes added to blob: {:?}\n", add_blob_bytes_outcome);

    // sleep(Duration::from_secs(3)).await;

    // let add_blob_named_outcome = add_blob_named(
    //     state.blobs.clone(), 
    //     "This is named blob".as_bytes(), 
    //     "named blob"
    // ).await?;
    // println!("Named blob added: {:?}\n", add_blob_named_outcome);

    // sleep(Duration::from_secs(3)).await;

    // let add_blob_from_path_outcome = add_blob_from_path(
    //     state.blobs.clone(),
    //     Path::new("_") // add absolute path to the file to upload
    // ).await?;
    // println!("Blob added from path: {:?}\n", add_blob_from_path_outcome);

    // sleep(Duration::from_secs(3)).await;

    // // // expect 3 blobs
    // let blobs = list_blobs(state.blobs.clone(), 0, 10).await?;
    // println!("List of blobs {} blobs found: ", blobs.len());
    // for blob in blobs {
    //     println!("{:?}", blob);
    // }
    // println!("\n");

    // sleep(Duration::from_secs(3)).await;

    // let blob_exists = has_blob(state.blobs.clone(), add_blob_bytes_outcome.hash.to_string()).await?;
    // println!("Blob exists: {:?}\n", blob_exists);

    // sleep(Duration::from_secs(3)).await;

    // let blob_content = get_blob(state.blobs.clone(), add_blob_bytes_outcome.hash.to_string()).await?;
    // println!("Blob content: {:?}\n", blob_content);

    // sleep(Duration::from_secs(3)).await;

    // let blob_status = status_blob(state.blobs.clone(), add_blob_bytes_outcome.hash.to_string()).await?;
    // println!("Blob status: {:?}\n", blob_status);

    // sleep(Duration::from_secs(3)).await;

    // let tags = list_tags(state.blobs.clone()).await?;
    // println!("List of tags {} tags found: ", tags.len());
    // for tag in tags {
    //     println!("{:?}", tag);
    // }
    // println!("\n");

    // sleep(Duration::from_secs(3)).await;

    // delete_tag(state.blobs.clone(), "named blob").await?;
    // println!("Tag deleted\n");

    // sleep(Duration::from_secs(3)).await;

    // let tags = list_tags(state.blobs.clone()).await?;
    // println!("List of tags {} tags found: ", tags.len());
    // for tag in tags {
    //     println!("{:?}", tag);
    // }
    // println!("\n");

    // sleep(Duration::from_secs(3)).await;

    // // expect 3 blobs
    // let blobs = list_blobs(state.blobs.clone(), 0, 10).await?;
    // println!("List of blobs {} blobs found: ", blobs.len());
    // for blob in blobs {
    //     println!("{:?}", blob);
    // }
    // println!("\n");

    // sleep(Duration::from_secs(3)).await;

    // let destination = PathBuf::from("_"); // add absolute path to the file to export
    // export_blob_to_file(state.blobs.clone(), add_blob_from_path_outcome.hash.to_string(), destination).await?;
    // println!("Blob exported to file\n");

    // sleep(Duration::from_secs(3)).await;

    // let download_options: DownloadOptions = DownloadOptions {
    //     format: iroh_blobs::BlobFormat::Raw,
    //     nodes: vec![iroh::NodeAddr::from(iroh_node.node_id)],
    //     tag: SetTagOption::Auto,
    //     mode: iroh_blobs::net_protocol::DownloadMode::Direct,
    // };
    // let download_with_options_outcome = download_with_options(
    //     state.blobs.clone(),
    //     add_blob_named_outcome.hash.to_string(),
    //     download_options
    // ).await?;
    // println!("Blob downloaded with options: {:?}\n", download_with_options_outcome);


    // Test scenario 4: Blob Store - Download(covers download_blob, download_hash_sequence)
    // for this test, first run test scenario 3 in a terminal, then after you get the line 'Server started on http://localhost:4000', comment it all. Then uncomment test scenario 4 and run it in a new terminal. You will have to pick up the nodeId from first terminal and put it in test scenario 4.

    // let node_id = "pass your nodeId"; // replace with the nodeId from first terminal
    // let blob_hash = "pass your blob hash"; // replace with the blob hash from first terminal
    // let download_blob_outcome: DownloadOutcome = download_blob(
    //     state.blobs.clone(),
    //     blob_hash.to_string(),
    //     node_id.to_string(),
    // ).await?;
    // println!("Blob downloaded: {:?}", download_blob_outcome);

    // let download_hash_sequence_outcome: DownloadOutcome = download_hash_sequence(
    //     state.blobs.clone(),
    //     blob_hash.to_string(),
    //     node_id.to_string(),
    // ).await?;
    // println!("Hash sequence downloaded: {:?}", download_hash_sequence_outcome);

    // Test scenario 5: Policy + Metadata + Cleanup(covers set_download_policy, get_download_policy, status, leave, drop_doc)

    // let doc_id = create_doc(state.docs.clone()).await?;
    // println!("Document created with ID: {:?}", doc_id);

    // let download_policy = json!({
    //     "policy": "nothing_except",
    //     "filters": []
    // });

    // set_download_policy(
    //     state.docs.clone(),
    //     doc_id.to_string(),
    //     download_policy,
    // ).await?;
    // println!("Download policy set for document: {:?}\n", doc_id);

    // sleep(Duration::from_secs(3)).await;

    // let policy = get_download_policy(
    //     state.docs.clone(),
    //     doc_id.to_string(),
    // ).await?;
    // println!("Download policy for document: {:?}\n", policy);

    // sleep(Duration::from_secs(3)).await;

    // let status = status(
    //     state.docs.clone(),
    //     doc_id.to_string(),
    // ).await?;
    // println!("Document status: {:?}\n", status);

    // sleep(Duration::from_secs(3)).await;

    // let leave = leave(
    //     state.docs.clone(),
    //     doc_id.to_string(),
    // ).await?;
    // println!("Document left: {:?}\n", leave);

    // println!("Documents before dropping: \n");
    // let docs = list_docs(state.docs.clone()).await?;
    // for doc in docs {
    //     println!("{:?}", doc);
    // }
    // println!("\n");

    // drop_doc(
    //     state.docs.clone(),
    //     doc_id.to_string(),
    // ).await?;
    // println!("Document dropped: {:?}\n", doc_id);

    // println!("Documents after dropping: \n");
    // let docs = list_docs(state.docs.clone()).await?;
    // for doc in docs {
    //     println!("{:?}", doc);
    // }
    // println!("\n");

    // Test scenario 6: Multiple Docs(covers list_docs, close_doc, share_doc)
    // let doc_id_1 = create_doc(state.docs.clone()).await?;
    // println!("Document created with ID: {:?}", doc_id_1);

    // sleep(Duration::from_secs(3)).await;

    // let doc_id_2 = create_doc(state.docs.clone()).await?;
    // println!("Document created with ID: {:?}", doc_id_2);

    // sleep(Duration::from_secs(3)).await;

    // let doc_id_3 = create_doc(state.docs.clone()).await?;
    // println!("Document created with ID: {:?}", doc_id_3);

    // sleep(Duration::from_secs(3)).await;

    // let docs = list_docs(state.docs.clone()).await?;
    // println!("Documents before closing: \n");
    // for doc in docs {
    //     println!("{:?}", doc);
    // }
    // println!("\n");

    // sleep(Duration::from_secs(3)).await;   

    // close_doc(
    //     state.docs.clone(),
    //     doc_id_1,
    // ).await?;

    // sleep(Duration::from_secs(3)).await;   

    // let docs = list_docs(state.docs.clone()).await?;
    // println!("Documents after closing: \n");
    // for doc in docs {
    //     println!("{:?}", doc);
    // }
    // println!("\n");

    // sleep(Duration::from_secs(3)).await;   

    // let ticket = share_doc(
    //     state.docs.clone(),
    //     doc_id_2,
    //     ShareMode::Read,
    //     AddrInfoOptions::Addresses,
    // ).await?;
    // println!("Document shared with ticket: {:?}", ticket);

    // Test scenario 7: join docs(covers join_doc)
    // for this test, first run test scenario 6 in a terminal, then after you get the line 'Server started on http://localhost:4000', comment it all. Then uncomment test scenario 7 and run it in a new terminal. You will have to pick up the ticket from first terminal and put it in test scenario 4.
    // let ticket = "pass the ticket here"; // replace with the ticket from first terminal
    // let doc_id = join_doc(
    //     state.docs.clone(), 
    //     ticket.to_string(),
    // ).await?;
    // println!("Document joined with ID: {:?}", doc_id);

//     // Test scenario 8: Authors(covers list_authors, get_default_author, set_default_author, create_author, delete_author, verify_author)
//     let authors = list_authors(state.docs.clone()).await?;
//     for author in authors {
//         println!("Initial authors: {:?}", author);
//     }
//     println!("\n");

//     sleep(Duration::from_secs(3)).await;

//     let new_author_1 = create_author(state.docs.clone()).await?;
//     println!("New author created: {:?}", new_author_1);

//     sleep(Duration::from_secs(3)).await;

//     let authors = list_authors(state.docs.clone()).await?;
//     for author in authors {
//         println!("Authors after adding a new author: {:?}", author);
//     }
//     println!("\n");

//     sleep(Duration::from_secs(3)).await;

//     set_default_author(state.docs.clone(), new_author_1.to_string()).await?;
//     println!("Default author set: {:?}", new_author_1);

//     sleep(Duration::from_secs(3)).await;

//     let default_author = get_default_author(state.docs.clone()).await?;
//     println!("Default author: {:?}", default_author);

//     sleep(Duration::from_secs(3)).await;    

//     let new_author_2 = create_author(state.docs.clone()).await?;
//     println!("New author created: {:?}", new_author_2);

//     sleep(Duration::from_secs(3)).await;

//     let new_author_3 = create_author(state.docs.clone()).await?;
//     println!("New author created: {:?}", new_author_3);

//     sleep(Duration::from_secs(3)).await;

//     let authors = list_authors(state.docs.clone()).await?;
//     for author in authors {
//         println!("Updated author list: {:?}", author);
//     }
//     println!("\n");

//     sleep(Duration::from_secs(3)).await;

//     delete_author(state.docs.clone(), new_author_3.to_string()).await?;
//    println!("Author deleted: {:?}", new_author_3);

//     sleep(Duration::from_secs(3)).await;

//     let authors = list_authors(state.docs.clone()).await?;
//     for author in authors {
//         println!("Authors list after deletion: {:?}", author);
//     }
//     println!("\n");

//     sleep(Duration::from_secs(3)).await;

//     let verify_author = verify_author(state.docs.clone(), new_author_2.to_string()).await?;
//     println!("Author verified: {}", verify_author);

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
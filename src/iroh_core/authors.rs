use anyhow::{Result, Context};
use std::{collections::HashSet, sync::Arc};
use iroh_docs::{protocol::Docs, AuthorId};
// use iroh_blobs::store::mem::Store as Store;
use iroh_blobs::store::fs::Store;
use futures::TryStreamExt;
use crate::helpers::utils::SS58AuthorId;

/// Lists all authors registered in the current context.
///
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
///
/// # Returns
/// * `Vec<String>` - A list of SS58-encoded author IDs.
pub async fn list_authors(
    docs: Arc<Docs<Store>>,
) -> Result<Vec<String>> {
    let authors_client = docs.client().authors();

    let mut author_stream = authors_client
        .list()
        .await
        .with_context(|| "Failed to list authors")?;

    let mut authors = Vec::new();

    while let Some(author) = author_stream
        .try_next()
        .await
        .with_context(|| "Error while streaming author list")?
    {
        let encode_author = SS58AuthorId::from_author_id(&author)?;
        authors.push(encode_author.as_ss58().to_string());
    }

    Ok(authors)
}

/// Retrieves the default author for the current Docs client.
///
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
///
/// # Returns
/// * `String` - The SS58-encoded ID of the default author.
pub async fn get_default_author(
    docs: Arc<Docs<Store>>,
) -> Result<String> {
    let authors_client = docs.client().authors();

    let default_author = authors_client
        .default()
        .await
        .with_context(|| "Failed to get default author")?;

    let encode_author = SS58AuthorId::from_author_id(&default_author)?;
    
    Ok(encode_author.as_ss58().to_string())
}

/// Sets the given author ID as the default author.
///
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
/// * `author_id` - The SS58-encoded ID of the author to set as default.
///
/// # Returns
/// * `()` - Returns unit on success.
pub async fn set_default_author(
    docs: Arc<Docs<Store>>,
    author_id: String
) -> Result<()> {
    let authors_client = docs.client().authors();

    let author = SS58AuthorId::decode(&author_id)?;

    authors_client
        .set_default(author)
        .await
        .with_context(|| "Failed to set default author")?;

    Ok(())
}

/// Creates a new author and returns its ID.
///
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
///
/// # Returns
/// * `String` - The SS58-encoded ID of the newly created author.
pub async fn create_author(
    docs: Arc<Docs<Store>>,
) -> Result<String> {
    let authors_client = docs.client().authors();

    let author_id = authors_client
        .create()
        .await
        .with_context(|| "Failed to create author")?;

    let encode_author = SS58AuthorId::from_author_id(&author_id)?;
    Ok(encode_author.as_ss58().to_string())
}

/// Deletes an author based on its ID.
///
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
/// * `author_id` - The SS58-encoded ID of the author to delete.
///
/// # Returns
/// * `()` - Returns unit on successful deletion.
pub async fn delete_author(
    docs: Arc<Docs<Store>>,
    author_id: String
) -> Result<()> {
    let authors_client = docs.client().authors();

    let author = SS58AuthorId::decode(&author_id)?;

    authors_client
        .delete(author)
        .await
        .with_context(|| "Failed to delete author")?;

    Ok(())
}

/// Verifies whether a given author ID exists.
///
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
/// * `author_id` - The SS58-encoded ID of the author to verify.
///
/// # Returns
/// * `bool` - True if the author exists, false otherwise.
pub async fn verify_author(
    docs: Arc<Docs<Store>>,
    author_id: String
) -> Result<bool> {
    let authors_client = docs.client().authors();

    let author = SS58AuthorId::decode(&author_id)?;

    let authors_set: HashSet<AuthorId> = 
        authors_client
            .list()
            .await
            .with_context(|| "Failed to list authors")?
            .try_collect::<HashSet<_>>()
            .await
            .with_context(|| "Error while collecting author list")?;

    Ok(authors_set.contains(&author))
}

mod tests {
    use super::*;
    use crate::node::iroh_wrapper::{
        setup_iroh_node,
        IrohNode};
    use crate::helpers::cli::CliArgs;
    use anyhow::{anyhow, Result};
    use std::default;
    use std::path::PathBuf;
    use tokio::fs;
    use tokio::time::{sleep, Duration};
    use tokio::process::Command;
    use std::process::Stdio;

    // Running tests will give any user understanding of how they should run the program in real life. 
    // step 1 is to run ```cargo run``` and fetch 'secret-key' form it and paste it in setup_node function.
    // step 2 is to run ```cargo run -- --path <path> --secret-key <your_secret_key>``` as this will create the data path and save the secret key in the data path. The test does this for user.
    // step 3 is to actually run the tests, but running it with ```cargo test``` will not work as all the tests will run in parallel and they will not be able to share the resources. Hence run the tests using ````cargo test -- --test-threads=1```.
    // If you wish to generate a lcov report, use ```cargo llvm-cov --html --tests -- --test-threads=1 --nocapture```.
    // To view the lcov file in browser, use ```open target/llvm-cov/html/index.html```.

    pub async fn setup_node() -> Result<IrohNode> {
        let secret_key = "cb9ce6327139d4d168ba753e4b12434f523221612fcabc600cdc57bba40c29de";

        fs::create_dir_all("Test").await?;

        let mut child = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--path")
        .arg("Test/test_blobs")
        .arg("--secret-key")
        .arg(secret_key)
        .stdout(Stdio::null()) // Silence output, or use `inherit()` for debug
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start cargo run");

        sleep(Duration::from_secs(5)).await;

        child.kill().await.ok();

        let args = CliArgs {
            path: Some(PathBuf::from("Test/test_blobs")),
            secret_key: Some(secret_key.to_string()), // remove this secret key
        };
        let iroh_node: IrohNode = setup_iroh_node(args).await.or_else(|_| {
            Err(anyhow!("Failed to set up Iroh node"))
        })?;
        println!("Iroh node started!");
        println!("Your NodeId: {}", iroh_node.node_id);
        Ok(iroh_node)
    }

    pub async fn delete_all_authors(docs: Arc<Docs<Store>>) -> Result<()> {
        let authors = list_authors(docs.clone()).await?;
        let default_author = get_default_author(docs.clone()).await?;

        for author in authors {
            if author == default_author {
                continue;
            }
            delete_author(docs.clone(), author).await?;
        }

        Ok(())
    }

    // create_author
    #[tokio::test]
    pub async fn test_create_author() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author_id = create_author(docs.clone()).await?;
        
        let authors = list_authors(docs.clone()).await?;
        assert!(authors.contains(&author_id));

        delete_all_authors(docs).await?;

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // list_authors
    #[tokio::test]
    pub async fn test_list_authors() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author_1 = create_author(docs.clone()).await?;
        let author_2 = create_author(docs.clone()).await?;
        let author_3 = create_author(docs.clone()).await?;
        
        let authors = list_authors(docs.clone()).await?;
        assert_eq!(authors.len(), 4); // 3 authors + default author
        assert!(authors.contains(&author_1));
        assert!(authors.contains(&author_2));
        assert!(authors.contains(&author_3));

        delete_all_authors(docs).await?;

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_list_authors_streaming_error() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        // Manually drop the router to simulate disconnection
        iroh_node.router.shutdown().await?;

        // Attempting to stream authors after shutting down router should fail
        let result = list_authors(docs.clone()).await;
        let error_str = format!("{:?}", result.unwrap_err());

        assert!(
            error_str.contains("Error while streaming author list"),
            "Expected streaming error, got: {}",
            error_str
        );

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;

        Ok(())
    }

    // get_default_author
    #[tokio::test]
    pub async fn test_get_default_author() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let default_author = get_default_author(docs.clone()).await?;
        let authors = list_authors(docs.clone()).await?;
        assert!(authors.contains(&default_author));
        assert_eq!(default_author, authors[0]);
        assert_eq!(authors.len(), 1);

        delete_all_authors(docs).await?;

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // set_default_author
    #[tokio::test]
    pub async fn test_set_default_author() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author_1 = create_author(docs.clone()).await?;
        sleep(Duration::from_secs(1)).await;
        
        let authors = list_authors(docs.clone()).await?;
        sleep(Duration::from_secs(1)).await;
        assert_eq!(authors.len(), 2); // 1 author + default author
        assert!(authors.contains(&author_1));

        let default_author = get_default_author(docs.clone()).await?;
        sleep(Duration::from_secs(1)).await;

        set_default_author(docs.clone(), author_1.clone()).await?;
        sleep(Duration::from_secs(1)).await;

        let new_default_author = get_default_author(docs.clone()).await?;
        sleep(Duration::from_secs(1)).await;
        assert_eq!(new_default_author, author_1);
        assert_ne!(default_author, new_default_author);

        delete_all_authors(docs).await?;

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // delete_author
    #[tokio::test]
    pub async fn test_delete_author() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author_id = create_author(docs.clone()).await?;
        sleep(Duration::from_secs(1)).await;

        let authors = list_authors(docs.clone()).await?;
        assert!(authors.contains(&author_id));
        sleep(Duration::from_secs(1)).await;

        delete_author(docs.clone(), author_id.clone()).await?;
        sleep(Duration::from_secs(1)).await;

        let authors = list_authors(docs.clone()).await?;
        assert!(!authors.contains(&author_id));
        sleep(Duration::from_secs(1)).await;

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // verify_author
    #[tokio::test]
    pub async fn test_verify_author() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let author_id = create_author(docs.clone()).await?;
        sleep(Duration::from_secs(1)).await;

        let verified = verify_author(docs.clone(), author_id.clone()).await?;
        assert!(verified);
        sleep(Duration::from_secs(1)).await;

        delete_author(docs.clone(), author_id.clone()).await?;
        sleep(Duration::from_secs(1)).await;

        let verified = verify_author(docs.clone(), author_id.clone()).await?;
        assert!(!verified);
        sleep(Duration::from_secs(1)).await;

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }

    // delete_all_authors
    #[tokio::test]
    pub async fn test_delete_all_authors() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let default_author = get_default_author(docs.clone()).await?;
        sleep(Duration::from_secs(1)).await;

        let author_1 = create_author(docs.clone()).await?;
        sleep(Duration::from_secs(1)).await;
        let author_2 = create_author(docs.clone()).await?;
        sleep(Duration::from_secs(1)).await;

        let authors = list_authors(docs.clone()).await?;
        assert!(authors.contains(&author_1));
        assert!(authors.contains(&author_2));
        assert!(authors.contains(&default_author));
        sleep(Duration::from_secs(1)).await;

        delete_all_authors(docs.clone()).await?;
        sleep(Duration::from_secs(1)).await;

        let authors = list_authors(docs.clone()).await?;
        assert!(!authors.contains(&author_1));
        assert!(!authors.contains(&author_2));
        assert!(authors.contains(&default_author));
        sleep(Duration::from_secs(1)).await;

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }
}
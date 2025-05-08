use anyhow::{Result, Context};
use std::{collections::HashSet, sync::Arc};
use iroh_docs::{protocol::Docs, AuthorId};
// use iroh_blobs::store::mem::Store as BlobStore;
use iroh_blobs::store::fs::Store as BlobStore;
use futures::TryStreamExt;
use crate::utils::SS58AuthorId;

// list the authors
pub async fn list_authors(
    docs: Arc<Docs<BlobStore>>,
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

// get the default author
pub async fn get_default_author(
    docs: Arc<Docs<BlobStore>>,
) -> Result<String> {
    let authors_client = docs.client().authors();

    let default_author = authors_client
        .default()
        .await
        .with_context(|| "Failed to get default author")?;

    let encode_author = SS58AuthorId::from_author_id(&default_author)?;
    
    Ok(encode_author.as_ss58().to_string())
}

// set the default author
pub async fn set_default_author(
    docs: Arc<Docs<BlobStore>>,
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

// create an author
pub async fn create_author(
    docs: Arc<Docs<BlobStore>>,
) -> Result<String> {
    let authors_client = docs.client().authors();

    let author_id = authors_client
        .create()
        .await
        .with_context(|| "Failed to create author")?;

    let encode_author = SS58AuthorId::from_author_id(&author_id)?;
    Ok(encode_author.as_ss58().to_string())
}

// delete an author
pub async fn delete_author(
    docs: Arc<Docs<BlobStore>>,
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

// verify an author
pub async fn verify_author(
    docs: Arc<Docs<BlobStore>>,
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
    use crate::iroh_wrapper::{
        setup_iroh_node,
        IrohNode};
    use crate::cli::CliArgs;
    use anyhow::{anyhow, Result};
    use std::default;
    use std::path::PathBuf;
    use tokio::fs;
    use tokio::time::{sleep, Duration};

    pub async fn setup_node() -> Result<IrohNode> {
        fs::create_dir_all("Test").await?;

        let args = CliArgs {
            path: Some(PathBuf::from("Test/test_blobs")),
            secret_key: Some("c6135803322e8c268313574920853c7f940489a74bee4d7e2566b773386283f2".to_string()), // remove this secret key
        };
        let iroh_node: IrohNode = setup_iroh_node(args).await.or_else(|_| {
            Err(anyhow!("Failed to set up Iroh node"))
        })?;
        println!("Iroh node started!");
        println!("Your NodeId: {}", iroh_node.node_id);
        Ok(iroh_node)
    }

    pub async fn delete_all_authors(docs: Arc<Docs<BlobStore>>) -> Result<()> {
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

        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;

        Ok(())
    }
}
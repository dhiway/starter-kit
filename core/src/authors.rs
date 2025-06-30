use helpers::utils::SS58AuthorId;

use anyhow::{Result, Context};
use std::{collections::HashSet, sync::Arc, fmt};
use iroh_docs::{protocol::Docs, AuthorId};
use iroh_blobs::store::fs::Store;
use futures::TryStreamExt;

// Errors
#[derive(Debug, PartialEq, Clone)]
pub enum AuthorError {
    /// The specified author was not found in the system.
    AuthorNotFound,
    /// No default author is set or the default author could not be found.
    DefaultAuthorNotFound,
    /// The provided author ID format is invalid or cannot be decoded.
    InvalidAuthorIdFormat,
    /// Failed to retrieve the list of authors from the backend.
    FailedToListAuthors,
    /// Failed to create a new author.
    FailedToCreateAuthor,
    /// Failed to delete the specified author.
    FailedToDeleteAuthor,
    /// Failed to set the specified author as the default.
    FailedToSetDefaultAuthor,
    /// An error occurred while streaming the list of authors.
    StreamingError,
    /// Failed to collect the authors from the stream.
    FailedToCollectAuthors,
}

impl fmt::Display for AuthorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AuthorError {}

/// Lists all authors registered in the current context.
///
/// # Arguments
/// * `docs` - The Arc-wrapped Docs client.
///
/// # Returns
/// * `Vec<String>` - A list of SS58-encoded author IDs.
pub async fn list_authors(
    docs: Arc<Docs<Store>>,
) -> Result<Vec<String>, AuthorError> {
    let authors_client = docs.client().authors();

    let mut author_stream = authors_client
        .list()
        .await
        .map_err(|_| AuthorError::FailedToListAuthors)?;

    let mut authors = Vec::new();

    while let Some(author) = author_stream
        .try_next()
        .await
        .map_err(|_| AuthorError::StreamingError)?
    {
        let encode_author = SS58AuthorId::from_author_id(&author)
            .map_err(|_| AuthorError::InvalidAuthorIdFormat)?;
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
) -> Result<String, AuthorError> {
    let authors_client = docs.client().authors();

    let default_author = authors_client
        .default()
        .await
        .map_err(|_| AuthorError::DefaultAuthorNotFound)?;

    let encode_author = SS58AuthorId::from_author_id(&default_author)
        .map_err(|_| AuthorError::InvalidAuthorIdFormat)?;
    
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
) -> Result<(), AuthorError> {
    let authors_client = docs.client().authors();

    let author = SS58AuthorId::decode(&author_id)
        .map_err(|_| AuthorError::InvalidAuthorIdFormat)?;

    authors_client
        .set_default(author)
        .await
        .map_err(|_| AuthorError::FailedToSetDefaultAuthor)?;

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
) -> Result<String, AuthorError> {
    let authors_client = docs.client().authors();

    let author_id = authors_client
        .create()
        .await
        .map_err(|_| AuthorError::FailedToCreateAuthor)?;

    let encode_author = SS58AuthorId::from_author_id(&author_id)
        .map_err(|_| AuthorError::InvalidAuthorIdFormat)?;
    
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
) -> Result<(), AuthorError> {
    let authors_client = docs.client().authors();

    let author = SS58AuthorId::decode(&author_id)
        .map_err(|_| AuthorError::InvalidAuthorIdFormat)?;
    
    // Check existence first
    let authors_set: HashSet<AuthorId> = 
        authors_client
            .list()
            .await
            .map_err(|_| AuthorError::FailedToListAuthors)?
            .try_collect::<HashSet<_>>()
            .await
            .map_err(|_| AuthorError::FailedToCollectAuthors)?;

    if !authors_set.contains(&author) {
        return Err(AuthorError::AuthorNotFound);
    }

    authors_client
        .delete(author)
        .await
        .map_err(|_| AuthorError::FailedToDeleteAuthor)?;

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
) -> Result<bool, AuthorError> {
    let authors_client = docs.client().authors();

    let author = SS58AuthorId::decode(&author_id)
        .map_err(|_| AuthorError::InvalidAuthorIdFormat)?;

    let authors_set: HashSet<AuthorId> = 
        authors_client
            .list()
            .await
            .map_err(|_| AuthorError::FailedToListAuthors)?
            .try_collect::<HashSet<_>>()
            .await
            .map_err(|_| AuthorError::FailedToCollectAuthors)?;

    Ok(authors_set.contains(&author))
}

mod tests {
    use super::*;
    use node::iroh_wrapper::{
        setup_iroh_node,
        IrohNode};
    use helpers::cli::CliArgs;

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
        if fs::try_exists("Test/test_blobs").await? {
            fs::remove_dir_all("Test/test_blobs").await?;
        }
        if fs::try_exists("Test").await? {
            fs::remove_dir_all("Test").await?;
        }

        sleep(Duration::from_secs(2)).await;

        fs::create_dir_all("Test").await?;

        let args = CliArgs {
            path: Some("Test/test_blobs".to_string()),
            password: "test_password".to_string(),
            bootstrap: true,
            suri: Some("0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a".to_string()), // don't use this suri in production, it is a preloaded suri for testing(for //Alice)
            secret: Some("test-secret".to_string()), // remove this secret key
        };
        let iroh_node: IrohNode = setup_iroh_node(args).await.or_else(|e| {
            Err(anyhow!("Failed to set up Iroh node. Error: {}", e))
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

        assert!(
            matches!(result, Err(AuthorError::StreamingError)),
            "Expected streaming error, got: {:?}",
            result
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

    // write a test to delete an author which does not exist
    #[tokio::test]
    pub async fn test_delete_non_existent_author() -> Result<()> {
        let iroh_node = setup_node().await?;
        let docs = iroh_node.docs.clone();

        let non_existent_author = "3uZsinKvBzw7MbhEo1F1Mmx8yWokz3E3cVfWGfrWvuHH8qFD".to_string();
        let result = delete_author(docs.clone(), non_existent_author).await;
        assert!(
            matches!(result, Err(AuthorError::AuthorNotFound)),
            "Expected AuthorNotFound error, got: {:?}",
            result
        );

        fs::remove_dir_all("Test/test_blobs").await?;
        fs::remove_dir_all("Test").await?;
        iroh_node.router.shutdown().await?;
        Ok(())
    }

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
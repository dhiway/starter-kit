use anyhow::{Result, Context};
use std::{collections::HashSet, sync::Arc};
use iroh_docs::{protocol::Docs, AuthorId};
use iroh_blobs::store::mem::Store as BlobStore;
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
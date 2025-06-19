use std::{collections::HashSet, path::PathBuf};
use tokio::fs;
use serde::{Serialize, Deserialize};
use anyhow;

#[derive(Serialize, Deserialize)]
struct PersistentData {
    node_ids: HashSet<String>,
    domains: HashSet<String>,
}

/// Initialize data directory and load access control lists
pub async fn init_access_control(path: &str) -> anyhow::Result<(HashSet<String>, HashSet<String>)> {
    let path = PathBuf::from(path);
    fs::create_dir_all(&path).await?;

    let node_ids = load_set(path.join("allowed_node_ids.json")).await.unwrap_or_default();
    let domains = load_set(path.join("allowed_domains.json")).await.unwrap_or_default();

    Ok((node_ids, domains))
}

/// Load a set from a JSON file
async fn load_set(file: PathBuf) -> anyhow::Result<HashSet<String>> {
    if !file.exists() {
        return Ok(HashSet::new());
    }

    let content = fs::read_to_string(file).await?;
    let set: HashSet<String> = serde_json::from_str(&content)?;
    Ok(set)
}

/// Write a set to a JSON file
pub async fn save_set(path: &str, filename: &str, set: &HashSet<String>) -> anyhow::Result<()> {
    let file_path = PathBuf::from(path).join(filename);
    let json = serde_json::to_string_pretty(set)?;
    fs::write(file_path, json).await?;
    Ok(())
}

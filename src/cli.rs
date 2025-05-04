use clap::Parser;
use std::path::PathBuf;

/// Command-line arguments for running the starter kit. 
/// ```
/// # Run with default in-memory and random secret
/// cargo run
///
/// # Run with persistent path and generated secret
/// cargo run -- --path ./iroh_data

/// # Run with persistent path and custom secret key
/// cargo run -- --path <path> --secret-key <your_secret_key>
/// ```
#[derive(Parser, Debug)]
#[command(name = "Starter Kit")]
#[command(about = "A starter kit for data providers", long_about = None)]
pub struct CliArgs {
    /// Path to persist docs and blobs. If not provided, memory storage is used.
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Hex-encoded secret key (64 characters). If not provided, a new key is generated.
    #[arg(short, long)]
    pub secret_key: Option<String>,
}
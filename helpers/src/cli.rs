use clap::Parser;
use std::path::PathBuf;

// Command-line arguments for running the starter kit. 
// ```
// Run with default in-memory and random secret
// cargo run
//
// Run with persistent path and generated secret
// cargo run -- --path ./iroh_data

// # Run with persistent path and custom secret key
// cargo run -- --path <path> --secret-key <your_secret_key>
// ```
#[derive(Parser, Debug, Clone)]
#[command(name = "Starter Kit")]
#[command(about = "A starter kit for decentralized data providers", long_about = None)]
pub struct CliArgs {
    /// Directory to persistently store blobs and documents.
    ///
    /// If not provided, a 'data' directory will be created in the current working directory.
    #[arg( 
        long,
        value_name = "PATH",
        help = "Path to persistently store the blobs and docs."
    )]
    pub path: Option<String>,

    /// Password for your Starter Kit node (required).
    ///
    /// This password is needed for both bootstrapping and restarting the node.
    #[arg(
        long,
        value_name = "PASSWORD",
        help = "A password for your starter-kit."
    )]
    pub password: String,

    /// Bootstrap a new node (requires --suri).
    ///
    /// If set, a new data directory will be created and keypairs will be generated from the provided SURI.
    #[arg(
        long,
        help = "Bootstraps the node. Requires the --suri parameter."
    )]
    pub bootstrap: bool,

    /// Seed phrase or SURI (Secret URI) to generate keypairs.
    ///
    /// Required when bootstrapping a new node. Only the SURI and public key are stored; the private key is derived at runtime.
    #[arg(
        long, 
        value_name = "SURI",
        help = "Seed phrase or SURI(secret URI) to generate keypairs."
    )]
    pub suri: Option<String>,

    /// Secret key for encrypting keypairs (optional, but recommended).
    ///
    /// Adds an extra layer of security for your keypairs. If provided, keypairs will be encrypted on disk.
    /// If the user uses secret to bootstrap the node, then he will be required to pass it again on restart.
    #[arg(
        long,
        value_name = "SECRET",
        help = "Added layer of security for your keypairs. If provided, the keypairs will get encrypted."
    )]
    pub secret: Option<String>,
}
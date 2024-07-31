use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod extract;

/// Extract the Candid interface from a Canister WASM file.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Path to the Canister WASM file.
    path: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let candid = extract::extract_candid(cli.path)?;
    println!("{candid}");
    Ok(())
}

use clap::Parser;
use std::path::PathBuf;
use anyhow::Result;

use candid_checker::check_rust;

#[derive(Parser)]
#[command(version, about)]
struct Opts {
    /// Rust source file
    rust: PathBuf,
    /// Candid file
    candid: PathBuf,
    /// TOML config file
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    check_rust(&opts.rust, &opts.candid, &opts.config)?;
    Ok(())
}

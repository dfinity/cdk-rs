use anyhow::{bail, Result};

mod extract;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        // The first arg will the name of current binary.
        bail!("Expecting one argument: path to the canister WASM file");
    }
    let c = extract::extract_candid(args.last().unwrap())?;
    println!("{c}");
    Ok(())
}

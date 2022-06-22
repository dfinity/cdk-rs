use cargo_metadata::MetadataCommand;
use escargot::CargoBuild;
use std::path::PathBuf;

/// Builds a canister with the specified name from the current
/// package and returns the WebAssembly module.
pub fn cargo_build_canister(bin_name: &str) -> Vec<u8> {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    let cargo_toml_path = dir.join("Cargo.toml");

    let target_dir = MetadataCommand::new()
        .manifest_path(&cargo_toml_path)
        .no_deps()
        .exec()
        .expect("failed to run cargo metadata")
        .target_directory;

    // We use a different target path to stop the native cargo build
    // cache being invalidated every time we run this function
    let wasm_target_dir = target_dir.join("canister-build");

    let cargo_build = CargoBuild::new()
        .target("wasm32-unknown-unknown")
        .bin(bin_name)
        .args(["--profile", "canister-release"])
        .manifest_path(&cargo_toml_path)
        .target_dir(&wasm_target_dir);

    let binary = cargo_build
        .run()
        .expect("Cargo failed to compile the wasm binary");

    std::fs::read(binary.path()).unwrap_or_else(|e| {
        panic!(
            "failed to read compiled Wasm file from {}: {}",
            binary.path().display(),
            e
        )
    })
}

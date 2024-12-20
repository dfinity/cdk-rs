use cargo_metadata::MetadataCommand;
use pocket_ic::{PocketIc, PocketIcBuilder};
use std::path::PathBuf;
use std::process::Command;

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

    let mut cmd = Command::new("cargo");
    let target = match std::env::var("WASM64") {
        Ok(_) => {
            cmd.args([
                "+nightly",
                "build",
                "-Z",
                "build-std=std,panic_abort",
                "--target",
                "wasm64-unknown-unknown",
            ]);
            "wasm64-unknown-unknown"
        }
        Err(_) => {
            cmd.args(["build", "--target", "wasm32-unknown-unknown"]);
            "wasm32-unknown-unknown"
        }
    };

    let cmd = cmd.args([
        "--bin",
        bin_name,
        "--profile",
        "canister-release",
        "--manifest-path",
        &cargo_toml_path.to_string_lossy(),
        "--target-dir",
        wasm_target_dir.as_ref(),
    ]);

    cmd.output().expect("failed to compile the wasm binary");

    let wasm_path = wasm_target_dir
        .join(target)
        .join("canister-release")
        .join(bin_name)
        .with_extension("wasm");

    std::fs::read(&wasm_path).unwrap_or_else(|e| {
        panic!(
            "failed to read compiled Wasm file from {:?}: {}",
            &wasm_path, e
        )
    })
}

pub fn pocket_ic() -> PocketIc {
    PocketIcBuilder::new()
        .with_application_subnet()
        .with_nonmainnet_features(true)
        .with_ii_subnet()
        .build()
}

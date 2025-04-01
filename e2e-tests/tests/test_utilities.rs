use candid::utils::{ArgumentDecoder, ArgumentEncoder};
use candid::Principal;
use cargo_metadata::MetadataCommand;
use fd_lock::RwLock as FdLock;
use flate2::read::GzDecoder;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::{call_candid, PocketIc, PocketIcBuilder, RejectResponse};
use std::fs::{OpenOptions, Permissions};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;

/// Builds a canister with the specified name from the current
/// package and returns the WebAssembly module.
pub fn cargo_build_canister(bin_name: &str) -> Vec<u8> {
    static LOG_INIT: Once = Once::new();
    LOG_INIT.call_once(env_logger::init);
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

// The linter complains "function `update` is never used"
// because not EVERY test uses this function.
pub fn update<Input, Output>(
    env: &PocketIc,
    canister_id: Principal,
    method: &str,
    input: Input,
) -> Result<Output, RejectResponse>
where
    Input: ArgumentEncoder,
    Output: for<'a> ArgumentDecoder<'a>,
{
    call_candid(env, canister_id, RawEffectivePrincipal::None, method, input)
}

/// Creates a PocketIcBuilder with the base configuration for e2e tests.
///
/// The PocketIc server binary is cached for reuse.
pub fn pic_base() -> PocketIcBuilder {
    let pocket_ic_server = cache_pocket_ic_server();
    PocketIcBuilder::new()
        .with_server_binary(pocket_ic_server)
        .with_application_subnet()
        .with_nonmainnet_features(true)
}

fn cache_pocket_ic_server() -> PathBuf {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let cargo_toml_path = dir.join("Cargo.toml");
    let metadata = MetadataCommand::new()
        .manifest_path(&cargo_toml_path)
        .exec()
        .expect("failed to run cargo metadata");
    let e2e_tests_package = metadata
        .packages
        .iter()
        .find(|m| m.name == "ic-cdk-e2e-tests")
        .expect("ic-cdk-e2e-tests not found in Cargo.toml");
    let pocket_ic_tag = e2e_tests_package
        .dependencies
        .iter()
        .find(|d| d.name == "pocket-ic")
        .expect("pocket-ic not found in Cargo.toml")
        .source
        .as_ref()
        .expect("pocket-ic source not found in Cargo.toml")
        .split_once("tag=")
        .expect("`tag=` not found in pocket-ic source")
        .1;
    let target_dir = metadata.target_directory;
    let artifact_dir = target_dir.join("e2e-tests-artifacts");
    std::fs::create_dir_all(&artifact_dir).expect("failed to create artifact directory");
    let tag_path = artifact_dir.join("pocket-ic-tag");
    let mut tag_file = FdLock::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            // same handle used for both reading and writing - the truncation is done with set_len below
            .truncate(false)
            .open(&tag_path)
            .unwrap(),
    );
    let mut tag_file = tag_file.write().unwrap();
    let pocket_ic_server = artifact_dir.join("pocket-ic-server");
    let mut tag = String::new();
    if tag_file.read_to_string(&mut tag).is_ok()
        && tag == pocket_ic_tag
        && pocket_ic_server.exists()
    {
        return pocket_ic_server.into();
    }

    let uname_sys = match std::env::consts::OS {
        "macos" => "darwin",
        "linux" => "linux",
        other => panic!("unsupported OS: {}", other),
    };
    let url = format!(
        "https://github.com/dfinity/ic/releases/download/{pocket_ic_tag}/pocket-ic-x86_64-{uname_sys}.gz");
    let gz_bytes = reqwest::blocking::get(url)
        .expect("failed to download pocket-ic-server")
        .bytes()
        .expect("failed to get bytes of pocket-ic-server")
        .to_vec();
    let mut decoder = GzDecoder::new(&gz_bytes[..]);
    let mut decompressed_data = Vec::new();
    decoder
        .read_to_end(&mut decompressed_data)
        .expect("failed to decompress pocket-ic-server");
    std::fs::write(&pocket_ic_server, decompressed_data).expect("failed to write pocket-ic-server");
    let permissions = Permissions::from_mode(0o755); // Make the file executable
    std::fs::set_permissions(&pocket_ic_server, permissions).expect("failed to set permissions");
    tag_file
        .write_all(pocket_ic_tag.as_bytes())
        .expect("failed to write pocket-ic-tag");
    tag_file.set_len(pocket_ic_tag.len() as _).unwrap();
    pocket_ic_server.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pocket_ic() {
        let _pic = pic_base();
    }

    #[test]
    fn test_update() {
        let pic = pic_base().build();
        let canister_id = pic.create_canister();
        pic.add_cycles(canister_id, 2_000_000_000_000);
        pic.install_canister(
            canister_id,
            b"\x00asm\x01\x00\x00\x00".to_vec(),
            vec![],
            None,
        );
        assert!(update::<(), ()>(&pic, canister_id, "insert", ()).is_err());
    }

    #[test]
    fn test_cache_pocket_ic_server() {
        let pocket_ic_server = cache_pocket_ic_server();
        println!("{:?}", pocket_ic_server);
    }
}

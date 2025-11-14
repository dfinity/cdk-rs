use ic_cdk_bindgen::{Builder, Config};
use std::path::PathBuf;

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("Cannot find manifest dir"));
    let profile_rs = Config::new("profile_rs");
    let mut builder = Builder::new();
    builder.add(profile_rs);
    builder.build(Some(manifest_dir.join("declarations")));
}

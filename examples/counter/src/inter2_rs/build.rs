use candid_build::{Builder, Config};
use std::path::PathBuf;

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("Cannot find manifest dir"));
    let counter = Config::new("inter_mo");
    let mut builder = Builder::new();
    builder.add(counter);
    builder.build(Some(manifest_dir.join("declarations")));
}

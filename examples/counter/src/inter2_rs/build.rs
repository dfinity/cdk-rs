use ic_cdk_bindgen::{Builder, Config};
use std::path::PathBuf;

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("Cannot find manifest dir"));
    let inter_mo = Config::new("inter_mo");
    let mut builder = Builder::new();
    builder.add(inter_mo);
    builder.build(Some(manifest_dir.join("declarations")));
}

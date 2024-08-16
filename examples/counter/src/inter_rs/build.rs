use ic_cdk_bindgen::{Builder, Config};
use std::path::PathBuf;

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("Cannot find manifest dir"));
    let counter = Config::new("counter_mo");
    let mut builder = Builder::new();
    builder.add(counter);
    builder.build(Some(manifest_dir.join("declarations")));
}

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use candid::Principal;
use trybuild::TestCases;
use std::env;

#[test]
fn test_mgmt() {
    let mut path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    path.push("ic.rs");
    ic_cdk_codegen::process_file("ic.did", &path, Principal::from_str("aaaaa-aa").unwrap()).unwrap();
    OpenOptions::new().append(true).open(&path).unwrap().write_all(b"\nfn main() {}").unwrap();
    TestCases::new().pass(&path);
}
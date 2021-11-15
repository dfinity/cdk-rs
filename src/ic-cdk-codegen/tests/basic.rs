use std::fs;
use std::io::Write;
use std::str::FromStr;

use candid::{IDLProg, Principal};
use tempfile::NamedTempFile;
use trybuild::TestCases;

#[test]
fn test_mgmt() {
    let mut tempfile = NamedTempFile::new().unwrap();
    let bindings = ic_cdk_codegen::process(
        IDLProg::from_str(&fs::read_to_string("ic.did").unwrap()).unwrap(),
        Principal::from_str("aaaaa-aa").unwrap(),
    )
    .unwrap();
    tempfile
        .write_all(b"#![allow(nonstandard_style, unused_parens, )] fn main() {} ")
        .unwrap();
    write!(tempfile, "{}", bindings).unwrap();
    let path = tempfile.into_temp_path();
    TestCases::new().pass(&path);
}

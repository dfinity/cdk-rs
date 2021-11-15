use std::fs;
use std::io::Write;
use std::str::FromStr;

use candid::{IDLProg, Principal};
use tempfile::NamedTempFile;
use trybuild::TestCases;

const FAKE: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";

#[test]
fn test_mgmt() {
    build_shim(&fs::read_to_string("ic.did").unwrap(), "aaaaa-aa");
}

#[test]
fn test_recursion() {
    build_shim("type list = opt record { int; list }", FAKE);
}

fn build_shim(input: &str, principal: &str) {
    let mut tempfile = NamedTempFile::new().unwrap();
    let bindings = ic_cdk_codegen::process(
        IDLProg::from_str(input).unwrap(),
        Principal::from_str(principal).unwrap(),
    )
    .unwrap();
    tempfile
        .write_all(b"#![allow(nonstandard_style)] fn main() {} ")
        .unwrap();
    write!(tempfile, "{}", bindings).unwrap();
    let path = tempfile.into_temp_path();
    TestCases::new().pass(&path);
}

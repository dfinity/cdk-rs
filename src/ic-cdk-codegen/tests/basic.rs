use std::fs;
use std::io::Write;
use std::str::FromStr;

use candid::{IDLProg, Principal};
use proc_macro2::TokenStream;
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

#[test]
fn test_corecursion() {
    build_shim("type bar = opt record { foo }; type foo = bar;", FAKE)
}

#[test]
fn test_service_types() {
    build_shim(
        "type greeter = service { greet: (text) -> (text) query; }; type cb = func () -> ()",
        FAKE,
    );
}

#[test]
fn test_naming() {
    build_shim_validator(
        "type foo = func (record { variant { bar: opt record { null } } } ) -> ()",
        FAKE,
        |bindings| assert!(bindings.to_string().contains("_foo_arg0_field0_bar")),
    );
}

fn build_shim(input: &str, principal: &str) {
    build_shim_validator(input, principal, |_| ())
}

fn build_shim_validator(input: &str, principal: &str, checker: impl FnOnce(TokenStream)) {
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
    checker(bindings);
}

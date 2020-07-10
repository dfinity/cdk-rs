use ic_cdk::{reflection, storage};
use ic_cdk_macros::{init, query, update};
use std::collections::BTreeMap;

#[init]
fn init() {
    let owner = storage::get_mut::<Vec<u8>>();
    let caller = reflection::caller();
    owner.clone_from(&caller);
}

#[update]
fn store(path: String, contents: Vec<u8>) {
    let store = storage::get_mut::<BTreeMap<String, Vec<u8>>>();
    let owner = storage::get::<Vec<u8>>();

    if owner != &reflection::caller() {
        panic!("Store can only be set by the owner of the asset canister.");
    }

    store.insert(path, contents);
}

#[query]
fn retrieve(path: String) -> &'static Vec<u8> {
    let store = storage::get::<BTreeMap<String, Vec<u8>>>();

    match store.get(&path) {
        Some(content) => content,
        None => panic!("Path {} not found.", path),
    }
}

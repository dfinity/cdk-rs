use ic_cdk::storage;
use ic_cdk_macros::{init, query, update};
use ic_types::Principal;
use std::collections::BTreeMap;

#[init]
fn init() {
    // Because Principal does not implement default, we use Option<Principal>.
    let owner = storage::get_mut::<Option<Principal>>();
    *owner = Some(ic_cdk::api::caller());
}

fn check_owner() -> Result<(), String> {
    let owner = storage::get::<Option<Principal>>();
    if let Some(o) = owner {
        if o != &ic_cdk::api::caller() {
            return Err("Store can only be set by the owner of the asset canister.".to_string());
        }
    }
    Ok(())
}

#[update(guard = "check_owner")]
fn store(path: String, contents: Vec<u8>) {
    let store = storage::get_mut::<BTreeMap<String, Vec<u8>>>();
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

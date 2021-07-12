use ic_cdk::{storage, export::Principal};
use ic_cdk_macros::*;
use std::collections::{BTreeMap, BTreeSet};

type Users = BTreeSet<Principal>;
type Store = BTreeMap<String, Vec<u8>>;

#[init]
fn init() {
    let users = storage::get_mut::<Users>();
    users.insert(ic_cdk::api::caller());
}

fn is_user() -> Result<(), String> {
    let users = storage::get::<Users>();

    if users.contains(&ic_cdk::api::caller()) {
        Ok(())
    } else {
        Err("Store can only be set by the owner of the asset canister.".to_string())
    }
}

#[update(guard = "is_user")]
fn store(path: String, contents: Vec<u8>) {
    let store = storage::get_mut::<Store>();
    store.insert(path, contents);
}

#[query]
fn retrieve(path: String) -> &'static Vec<u8> {
    let store = storage::get::<Store>();

    match store.get(&path) {
        Some(content) => content,
        None => panic!("Path {} not found.", path),
    }
}

#[update(guard = "is_user")]
fn add_user(principal: Principal) {
    let users = storage::get_mut::<Users>();
    users.insert(principal);
}

#[pre_upgrade]
fn pre_upgrade() {
    let mut vec = Vec::new();
    for p in storage::get_mut::<Users>().iter() {
        vec.push(p);
    }
    storage::stable_save((vec,)).unwrap();
}

#[post_upgrade]
fn post_upgrade() {
    let (old_users,): (Vec<Principal>,) = storage::stable_restore().unwrap();
    for u in old_users {
        storage::get_mut::<Users>().insert(u);
    }
}

use ic_cdk::{
    api::call::ManualReply, export::Principal, init, post_upgrade, pre_upgrade, query, storage,
    update,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

type Users = BTreeSet<Principal>;
type Store = BTreeMap<String, Vec<u8>>;

thread_local! {
    static USERS: RefCell<Users> = RefCell::default();
    static STORE: RefCell<Store> = RefCell::default();
}

#[init]
fn init() {
    USERS.with(|users| users.borrow_mut().insert(ic_cdk::api::caller()));
}

fn is_user() -> Result<(), String> {
    if USERS.with(|users| users.borrow().contains(&ic_cdk::api::caller())) {
        Ok(())
    } else {
        Err("Store can only be set by the owner of the asset canister.".to_string())
    }
}

#[update(guard = "is_user")]
fn store(path: String, contents: Vec<u8>) {
    STORE.with(|store| store.borrow_mut().insert(path, contents));
}

#[query(manual_reply = true)]
fn retrieve(path: String) -> ManualReply<Vec<u8>> {
    STORE.with(|store| match store.borrow().get(&path) {
        Some(content) => ManualReply::one(content),
        None => panic!("Path {} not found.", path),
    })
}

#[update(guard = "is_user")]
fn add_user(principal: Principal) {
    USERS.with(|users| users.borrow_mut().insert(principal));
}

#[pre_upgrade]
fn pre_upgrade() {
    USERS.with(|users| storage::stable_save((users,)).unwrap());
}

#[post_upgrade]
fn post_upgrade() {
    let (old_users,): (BTreeSet<Principal>,) = storage::stable_restore().unwrap();
    USERS.with(|users| *users.borrow_mut() = old_users);
}

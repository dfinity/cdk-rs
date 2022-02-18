use ic_cdk::storage;
use ic_cdk::{
    api::call::{self, Empty},
    export::{
        candid::{CandidType, Deserialize},
        Principal,
    },
};
use ic_cdk_macros::*;
use std::collections::BTreeMap;

type IdStore = BTreeMap<String, Principal>;
type ProfileStore = BTreeMap<Principal, Profile>;

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct Profile {
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
}

#[query(name = "getSelf")]
fn get_self() -> Profile {
    let id = ic_cdk::api::caller();
    let profile_store = storage::get::<ProfileStore>();

    profile_store
        .get(&id)
        .cloned()
        .unwrap_or_else(|| Profile::default())
}

#[query]
fn get(name: String) -> Profile {
    let id_store = storage::get::<IdStore>();
    let profile_store = storage::get::<ProfileStore>();

    id_store
        .get(&name)
        .and_then(|id| profile_store.get(id).cloned())
        .unwrap_or_else(|| Profile::default())
}

#[update]
fn update(profile: Profile) {
    let principal_id = ic_cdk::api::caller();
    let id_store = storage::get_mut::<IdStore>();
    let profile_store = storage::get_mut::<ProfileStore>();

    id_store.insert(profile.name.clone(), principal_id.clone());
    profile_store.insert(principal_id, profile);
}

#[query(reply = true)]
fn search(text: String) -> Empty<Option<Profile>> {
    let text = text.to_lowercase();
    let profile_store = storage::get::<ProfileStore>();

    let mut profile = None;
    'outer: for (_, p) in profile_store.iter() {
        if p.name.to_lowercase().contains(&text) || p.description.to_lowercase().contains(&text) {
            profile = Some(p);
            break 'outer;
        }

        for x in p.keywords.iter() {
            if x.to_lowercase() == text {
                profile = Some(p);
                break 'outer;
            }
        }
    }

    call::reply((profile,));
    Empty::empty()
}

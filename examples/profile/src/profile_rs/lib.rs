use ic_cdk::{
    api::call::ManualReply,
    export::{
        candid::{CandidType, Deserialize},
        Principal,
    },
};
use ic_cdk_macros::*;
use std::cell::RefCell;
use std::collections::BTreeMap;

type IdStore = BTreeMap<String, Principal>;
type ProfileStore = BTreeMap<Principal, Profile>;

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct Profile {
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
}

thread_local! {
    static PROFILE_STORE: RefCell<ProfileStore> = RefCell::default();
    static ID_STORE: RefCell<IdStore> = RefCell::default();
}

#[query(name = "getSelf", manual_reply = true)]
fn get_self() -> ManualReply<Profile> {
    let id = ic_cdk::api::caller();
    PROFILE_STORE.with(|profile_store| {
        if let Some(profile) = profile_store.borrow().get(&id) {
            ManualReply::one(profile)
        } else {
            ManualReply::one(Profile::default())
        }
    })
}

#[query(manual_reply = true)]
fn get(name: String) -> ManualReply<Profile> {
    ID_STORE.with(|id_store| {
        PROFILE_STORE.with(|profile_store| {
            let profile_store = profile_store.borrow();
            if let Some(profile) = id_store
                .borrow()
                .get(&name)
                .and_then(|id| profile_store.get(id))
            {
                ManualReply::one(profile)
            } else {
                ManualReply::one(Profile::default())
            }
        })
    })
}

#[update]
fn update(profile: Profile) {
    let principal_id = ic_cdk::api::caller();
    ID_STORE.with(|id_store| {
        id_store
            .borrow_mut()
            .insert(profile.name.clone(), principal_id);
    });
    PROFILE_STORE.with(|profile_store| {
        profile_store.borrow_mut().insert(principal_id, profile);
    });
}

#[query(manual_reply = true)]
fn search(text: String) -> ManualReply<Option<Profile>> {
    let text = text.to_lowercase();
    PROFILE_STORE.with(|profile_store| {
        for (_, p) in profile_store.borrow().iter() {
            if p.name.to_lowercase().contains(&text) || p.description.to_lowercase().contains(&text)
            {
                return ManualReply::one(Some(p));
            }

            for x in p.keywords.iter() {
                if x.to_lowercase() == text {
                    return ManualReply::one(Some(p));
                }
            }
        }
        ManualReply::one(None::<Profile>)
    })
}

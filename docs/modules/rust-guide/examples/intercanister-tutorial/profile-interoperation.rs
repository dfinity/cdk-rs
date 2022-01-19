use ic_cdk_macros::*;

#[import(canister = "rust_profile")]
mod profile_canister {}
use profile_canister::{Profile, Profile_2};

#[update(name = "getSelf")]
async fn get_self() -> Profile_2 {
    profile_canister::getSelf().await
}

#[update]
async fn get(name: String) -> Profile_2 {
    profile_canister::get(name).await
}

#[update]
async fn update(profile: Profile) {
    profile_canister::update(profile).await
}

#[update]
async fn search(text: String) -> Option<Profile_2> {
    profile_canister::search(text).await
}

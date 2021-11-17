use ic_cdk_macros::*;

#[import(canister = "profile_rs")]
mod profile_canister {}

use profile_canister::Profile;

#[update(name = "getSelf")]
async fn get_self() -> Profile {
    profile_canister::getSelf().await
}

#[update]
async fn get(name: String) -> Profile {
    profile_canister::get(name).await
}

#[update]
async fn update(profile: Profile) {
    profile_canister::update(profile).await
}

#[update]
async fn search(text: String) -> Option<Profile> {
    profile_canister::search(text).await
}

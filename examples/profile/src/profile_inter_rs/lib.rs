use ic_cdk_macros::*;
use ic_cdk::export::candid;

#[import(canister = "profile_rs")]
struct ProfileCanister;

#[update(name = "getSelf")]
async fn get_self() -> Box<Profile_2> {
    ProfileCanister::getSelf().await.0
}

#[update]
async fn get(name: String) -> Box<Profile_2> {
    ProfileCanister::get(name).await.0
}

#[update]
async fn update(profile: Profile) {
    ProfileCanister::update(profile).await
}

#[update]
async fn search(text: String) -> Option<Box<Profile_2>> {
    ProfileCanister::search(text).await.0
}

use ic_cdk_macros::*;

#[import(canister = "profile_rs")]
struct ProfileCanister;

#[update(name = "getSelf")]
async fn get_self() -> Box<Profile> {
    ProfileCanister::getSelf().await.0
}

#[update]
async fn get(name: String) -> Box<Profile> {
    ProfileCanister::get(name).await.0
}

#[update]
async fn update(profile: Profile) {
    ProfileCanister::update(Box::new(profile)).await
}

#[update]
async fn search(text: String) -> Option<Box<Profile>> {
    ProfileCanister::search(text).await.0
}

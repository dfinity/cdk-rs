use ic_cdk_macros::*;

#[import(canister = "rust_profile")]
struct ProfileCanister;

#[update(name = "getSelf")]
async fn get_self() -> Box<Profile_2> {
    ProfileCanister::getSelf().await
}

#[update]
async fn get(name: String) -> Box<Profile_2> {
    ProfileCanister::get(name).await
}

#[update]
async fn update(profile: Profile) {
    ProfileCanister::update(profile).await
}

#[update]
async fn search(text: String) -> Option<Box<Profile_2>> {
    ProfileCanister::search(text).await
}

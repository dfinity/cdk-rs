use ic_cdk::update;

mod declarations;
use declarations::profile_rs::{profile_rs, Profile};

#[update(name = "getSelf")]
async fn get_self() -> Profile {
    profile_rs.get_self().await.unwrap().0
}

#[update]
async fn get(name: String) -> Profile {
    profile_rs.get(name).await.unwrap().0
}

#[update]
async fn update(profile: Profile) {
    profile_rs.update(profile).await.unwrap()
}

#[update]
async fn search(text: String) -> Option<Profile> {
    profile_rs.search(text).await.unwrap().0
}

ic_cdk::export_candid!();

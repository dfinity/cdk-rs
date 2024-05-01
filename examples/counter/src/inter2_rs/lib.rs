use ic_cdk::update;

mod inter_mo {
    include!(concat!(env!("OUT_DIR"), "/consumer/inter_mo.rs"));
}
use inter_mo::inter_mo;

#[update]
async fn read() -> candid::Nat {
    inter_mo.read().await.unwrap().0
}

#[update]
async fn inc() {
    inter_mo.inc().await.unwrap()
}

#[update]
async fn write(input: candid::Nat) {
    inter_mo.write(input).await.unwrap()
}

ic_cdk::export_candid!();

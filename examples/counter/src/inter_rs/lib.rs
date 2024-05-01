use ic_cdk::update;

mod counter_mo {
    include!(concat!(env!("OUT_DIR"), "/consumer/counter_mo.rs"));
}
use counter_mo::counter_mo;

#[update]
async fn read() -> candid::Nat {
    counter_mo.read().await.unwrap().0
}

#[update]
async fn inc() {
    counter_mo.inc().await.unwrap()
}

#[update]
async fn write(input: candid::Nat) {
    counter_mo.write(input).await.unwrap()
}

ic_cdk::export_candid!();

use ic_cdk::update;

mod declarations;
use declarations::counter_mo::counter_mo;

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

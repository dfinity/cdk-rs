#[ic_cdk_macros::update]
fn whoami() -> ic_cdk::export::Principal {
    ic_cdk::api::caller()
}

#[ic_cdk_macros::query]
fn whoami_query() -> ic_cdk::export::Principal {
    ic_cdk::api::caller()
}

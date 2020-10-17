use ic_cdk_macros::*;

struct Service {
    counter: candid::Nat,
}
impl Service {
    fn init() -> Self {
        Service { counter: 0.into() }
    }
    pub fn read(&self) -> &candid::Nat {
        &self.counter
    }
    pub fn inc(&mut self) -> () {
        self.counter += 1;
    }
}

static mut SERVICE: Option<Service> = None;

#[export_name = "canister_init"]
fn init_2_() {
    ic_cdk::setup();
    ic_cdk::block_on(async {
        unsafe { SERVICE = Some(Service::init()) };
    });
}

#[export_name = "canister_query read"]
fn read_0_() {
    ic_cdk::setup();
    ic_cdk::block_on(async {
        let () = ic_cdk::api::call::arg_data();
        let result = unsafe { SERVICE.as_mut().unwrap().read() };
        ic_cdk::api::call::reply((result,))
    });
}

#[export_name = "canister_update inc"]
fn inc_1_() {
    ic_cdk::setup();
    ic_cdk::block_on(async {
        let () = ic_cdk::api::call::arg_data();
        let result = unsafe { SERVICE.as_mut().unwrap().inc() };
        ic_cdk::api::call::reply(())
    });
}

use candid::{Encode, Principal};
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::{call_candid, query_candid, WasmResult};

mod test_utilities;
use test_utilities::{cargo_build_canister, pocket_ic};

#[test]
fn call_apis() {
    let pic = pocket_ic();
    let wasm = cargo_build_canister("api_call");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);
    let (result,): (u64,) = query_candid(&pic, canister_id, "instruction_counter", ())
        .expect("failed to query instruction_counter");
    assert!(result > 0);

    let result = pic
        .query_call(
            canister_id,
            Principal::anonymous(),
            "manual_reject",
            Encode!().unwrap(),
        )
        .unwrap();
    assert_eq!(result, WasmResult::Reject("manual reject".to_string()));

    let (result,): (bool,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "update_is_replicated",
        (),
    )
    .expect("Failed to call update_is_replicated");
    assert!(result);

    let (result,): (bool,) = query_candid(&pic, canister_id, "query_is_not_replicated", ())
        .expect("Failed to call query_is_not_replicated");
    assert!(!result);
}

#[test]
fn cycles_burn() {
    let pic = pocket_ic();
    let wasm = cargo_build_canister("api_call");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);

    pic.install_canister(canister_id, wasm, vec![], None);
    eprintln!("Canister installed.");
    let balance1 = pic.cycle_balance(canister_id);
    eprintln!("Balance 1: {balance1}");

    // Scenario 1: burn less than balance
    let attempted1 = 1000u128;
    let (burned,): (u128,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "cycles_burn",
        (attempted1,),
    )
    .expect("Error calling cycles_burn");
    eprintln!("Attempted to burn {attempted1}, actually burned {burned}");
    assert_eq!(burned, attempted1);
    let balance2 = pic.cycle_balance(canister_id);
    eprintln!("Balance 2: {balance2}");

    // Scenario 2: burn more than balance
    let attempted2 = balance2 + 1;
    let (burned,): (u128,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "cycles_burn",
        (attempted2,),
    )
    .expect("Error calling cycles_burn");
    eprintln!("Attempted to burn {attempted2}, actually burned {burned}");
    assert!(burned < balance2); // restrained by reserved_balance and freezing_limit
    let balance3 = pic.cycle_balance(canister_id);
    eprintln!("Balance 3: {balance3}");
}

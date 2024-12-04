use candid::Principal;
use ic_cdk::api::management_canister::main::{
    CanisterChange, CanisterChangeDetails, CanisterChangeOrigin, CanisterIdRecord,
    CanisterInfoResponse, CanisterInstallMode,
    CodeDeploymentMode::{Install, Reinstall, Upgrade},
    CodeDeploymentRecord, ControllersChangeRecord, CreationRecord, FromCanisterRecord,
    FromUserRecord, InstallCodeArgument,
};
use ic_cdk_e2e_tests::cargo_build_canister;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::{call_candid, call_candid_as, PocketIc};
use std::time::UNIX_EPOCH;

#[test]
fn test_canister_info() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("canister_info");
    // As of PocketIC server v5.0.0 and client v4.0.0, the first canister creation happens at (time0+4).
    // Each operation advances the Pic by 2 nanos, except for the last operation which advances only by 1 nano.
    let time0: u64 = pic
        .get_time()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .try_into()
        .unwrap();
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);

    let new_canister: (Principal,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "canister_lifecycle",
        (),
    )
    .expect("Error calling canister_lifecycle");

    let () = call_candid_as(
        &pic,
        Principal::management_canister(),
        RawEffectivePrincipal::None,
        Principal::anonymous(),
        "uninstall_code",
        (CanisterIdRecord {
            canister_id: new_canister.0,
        },),
    )
    .expect("Error calling uninstall_code");
    let () = call_candid_as(
        &pic,
        Principal::management_canister(),
        RawEffectivePrincipal::None,
        Principal::anonymous(),
        "install_code",
        (InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            arg: vec![],
            wasm_module: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
            canister_id: new_canister.0,
        },),
    )
    .expect("Error calling install_code");

    let info: (CanisterInfoResponse,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "info",
        (new_canister.0,),
    )
    .expect("Error calling canister_info");

    assert_eq!(
        info.0,
        CanisterInfoResponse {
            total_num_changes: 9,
            recent_changes: vec![
                CanisterChange {
                    timestamp_nanos: time0 + 4,
                    canister_version: 0,
                    origin: CanisterChangeOrigin::FromCanister(FromCanisterRecord {
                        canister_id,
                        canister_version: Some(1)
                    }),
                    details: CanisterChangeDetails::Creation(CreationRecord {
                        controllers: vec![canister_id]
                    }),
                },
                CanisterChange {
                    timestamp_nanos: time0 + 6,
                    canister_version: 1,
                    origin: CanisterChangeOrigin::FromCanister(FromCanisterRecord {
                        canister_id,
                        canister_version: Some(2)
                    }),
                    details: CanisterChangeDetails::CodeDeployment(CodeDeploymentRecord {
                        mode: Install,
                        module_hash: hex::decode(
                            "93a44bbb96c751218e4c00d479e4c14358122a389acca16205b1e4d0dc5f9476"
                        )
                        .unwrap(),
                    }),
                },
                CanisterChange {
                    timestamp_nanos: time0 + 8,
                    canister_version: 2,
                    origin: CanisterChangeOrigin::FromCanister(FromCanisterRecord {
                        canister_id,
                        canister_version: Some(3)
                    }),
                    details: CanisterChangeDetails::CodeUninstall,
                },
                CanisterChange {
                    timestamp_nanos: time0 + 10,
                    canister_version: 3,
                    origin: CanisterChangeOrigin::FromCanister(FromCanisterRecord {
                        canister_id,
                        canister_version: Some(4)
                    }),
                    details: CanisterChangeDetails::CodeDeployment(CodeDeploymentRecord {
                        mode: Install,
                        module_hash: hex::decode(
                            "93a44bbb96c751218e4c00d479e4c14358122a389acca16205b1e4d0dc5f9476"
                        )
                        .unwrap(),
                    }),
                },
                CanisterChange {
                    timestamp_nanos: time0 + 12,
                    canister_version: 4,
                    origin: CanisterChangeOrigin::FromCanister(FromCanisterRecord {
                        canister_id,
                        canister_version: Some(5)
                    }),
                    details: CanisterChangeDetails::CodeDeployment(CodeDeploymentRecord {
                        mode: Reinstall,
                        module_hash: hex::decode(
                            "93a44bbb96c751218e4c00d479e4c14358122a389acca16205b1e4d0dc5f9476"
                        )
                        .unwrap(),
                    }),
                },
                CanisterChange {
                    timestamp_nanos: time0 + 14,
                    canister_version: 5,
                    origin: CanisterChangeOrigin::FromCanister(FromCanisterRecord {
                        canister_id,
                        canister_version: Some(6)
                    }),
                    details: CanisterChangeDetails::CodeDeployment(CodeDeploymentRecord {
                        mode: Upgrade,
                        module_hash: hex::decode(
                            "93a44bbb96c751218e4c00d479e4c14358122a389acca16205b1e4d0dc5f9476"
                        )
                        .unwrap(),
                    }),
                },
                CanisterChange {
                    timestamp_nanos: time0 + 16,
                    canister_version: 6,
                    origin: CanisterChangeOrigin::FromCanister(FromCanisterRecord {
                        canister_id,
                        canister_version: Some(7)
                    }),
                    details: CanisterChangeDetails::ControllersChange(ControllersChangeRecord {
                        controllers: vec![Principal::anonymous(), canister_id, new_canister.0]
                    }),
                },
                CanisterChange {
                    timestamp_nanos: time0 + 18,
                    canister_version: 7,
                    origin: CanisterChangeOrigin::FromUser(FromUserRecord {
                        user_id: Principal::anonymous(),
                    }),
                    details: CanisterChangeDetails::CodeUninstall,
                },
                CanisterChange {
                    timestamp_nanos: time0 + 19,
                    canister_version: 8,
                    origin: CanisterChangeOrigin::FromUser(FromUserRecord {
                        user_id: Principal::anonymous(),
                    }),
                    details: CanisterChangeDetails::CodeDeployment(CodeDeploymentRecord {
                        mode: Install,
                        module_hash: hex::decode(
                            "93a44bbb96c751218e4c00d479e4c14358122a389acca16205b1e4d0dc5f9476"
                        )
                        .unwrap(),
                    }),
                },
            ],
            module_hash: Some(
                hex::decode("93a44bbb96c751218e4c00d479e4c14358122a389acca16205b1e4d0dc5f9476")
                    .unwrap()
            ),
            controllers: vec![Principal::anonymous(), canister_id, new_canister.0],
        }
    );
}

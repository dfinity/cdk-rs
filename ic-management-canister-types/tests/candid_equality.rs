#![allow(unused)]
use candid::candid_method;
use ic_management_canister_types::*;

#[candid_method(update)]
fn create_canister(_: CreateCanisterArgs) -> CreateCanisterResult {
    unreachable!()
}

#[candid_method(update)]
fn update_settings(_: UpdateSettingsArgs) {
    unreachable!()
}

#[candid_method(update)]
fn upload_chunk(_: UploadChunkArgs) -> UploadChunkResult {
    unreachable!()
}

#[candid_method(update)]
fn clear_chunk_store(_: ClearChunkStoreArgs) {
    unreachable!()
}

#[candid_method(update)]
fn stored_chunks(_: StoredChunksArgs) -> StoredChunksResult {
    unreachable!()
}

#[candid_method(update)]
fn install_code(_: InstallCodeArgs) {
    unreachable!()
}

#[candid_method(update)]
fn install_chunked_code(_: InstallChunkedCodeArgs) {
    unreachable!()
}

#[candid_method(update)]
fn uninstall_code(_: UninstallCodeArgs) {
    unreachable!()
}

#[candid_method(update)]
fn start_canister(_: StartCanisterArgs) {
    unreachable!()
}

#[candid_method(update)]
fn stop_canister(_: StopCanisterArgs) {
    unreachable!()
}

#[candid_method(update)]
fn canister_status(_: CanisterStatusArgs) -> CanisterStatusResult {
    unreachable!()
}

#[candid_method(update)]
fn canister_info(_: CanisterInfoArgs) -> CanisterInfoResult {
    unreachable!()
}

#[candid_method(update)]
fn subnet_info(_: SubnetInfoArgs) -> SubnetInfoResult {
    unreachable!()
}

#[candid_method(update)]
fn delete_canister(_: DeleteCanisterArgs) {
    unreachable!()
}

#[candid_method(update)]
fn deposit_cycles(_: DepositCyclesArgs) {
    unreachable!()
}

#[candid_method(update)]
fn raw_rand() -> RawRandResult {
    unreachable!()
}

#[candid_method(update)]
fn http_request(_: HttpRequestArgs) -> HttpRequestResult {
    unreachable!()
}

#[candid_method(update)]
fn ecdsa_public_key(_: EcdsaPublicKeyArgs) -> EcdsaPublicKeyResult {
    unreachable!()
}

#[candid_method(update)]
fn sign_with_ecdsa(_: SignWithEcdsaArgs) -> SignWithEcdsaResult {
    unreachable!()
}

#[candid_method(update)]
fn schnorr_public_key(_: SchnorrPublicKeyArgs) -> SchnorrPublicKeyResult {
    unreachable!()
}

#[candid_method(update)]
fn sign_with_schnorr(_: SignWithSchnorrArgs) -> SignWithSchnorrResult {
    unreachable!()
}

#[candid_method(update)]
fn node_metrics_history(_: NodeMetricsHistoryArgs) -> NodeMetricsHistoryResult {
    unreachable!()
}

#[candid_method(update)]
fn provisional_create_canister_with_cycles(
    _: ProvisionalCreateCanisterWithCyclesArgs,
) -> ProvisionalCreateCanisterWithCyclesResult {
    unreachable!()
}

#[candid_method(update)]
fn provisional_top_up_canister(_: ProvisionalTopUpCanisterArgs) {
    unreachable!()
}

#[candid_method(update)]
fn take_canister_snapshot(_: TakeCanisterSnapshotArgs) -> TakeCanisterSnapshotResult {
    unreachable!()
}

#[candid_method(update)]
fn load_canister_snapshot(_: LoadCanisterSnapshotArgs) {
    unreachable!()
}

#[candid_method(update)]
fn list_canister_snapshots(_: ListCanisterSnapshotsArgs) -> ListCanisterSnapshotsResult {
    unreachable!()
}

#[candid_method(update)]
fn delete_canister_snapshot(_: DeleteCanisterSnapshotArgs) {
    unreachable!()
}

#[candid_method(query)]
fn fetch_canister_logs(_: FetchCanisterLogsArgs) -> FetchCanisterLogsResult {
    unreachable!()
}

#[cfg(test)]
mod test {
    use candid_parser::utils::{service_equal, CandidSource};
    use ic_management_canister_types::*;

    #[test]
    fn candid_equality_test() {
        let declared_interface_str =
            std::fs::read_to_string("tests/ic.did").expect("Failed to read ic.did file");
        let declared_interface = CandidSource::Text(&declared_interface_str);

        candid::export_service!();
        let implemented_interface_str = __export_service();
        let implemented_interface = CandidSource::Text(&implemented_interface_str);

        let result = service_equal(declared_interface, implemented_interface);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
    }
}

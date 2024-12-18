/// Create a `get_candid_pointer` method so that `dfx` can execute it to extract candid definition.
///
/// Call this macro only if you want the Candid export behavior.
/// Only call it once at the end of canister code outside query/update definition.
pub use ic_cdk_macros::export_candid;

/// Register a query call entry point.
///
/// This attribute macro will export a function with name `canister_query <name>`
/// in the canister module.
///
/// # Example
///
/// ```rust
/// # use ic_cdk::query;
/// #[query]
/// fn query_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
///
/// You can also specify the name of the exported function.
///
/// ```rust
/// # use ic_cdk::query;
/// #[query(name = "some_name")]
/// fn query_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
///
/// If you want to hide this method in the Candid generated by [export_candid!],
/// you will need to set `hidden` to `true`. The entry point still exists in the canister.
///
/// ```rust
/// # use ic_cdk::query;
/// #[query(hidden = true)]
/// fn query_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
///
/// You can specify a guard function to be executed before the query function.
/// When the guard function returns an error, the query function will not proceed.
///
/// ```rust
/// # use ic_cdk::query;
/// fn guard_function() -> Result<(), String> {
///     // ...
/// # unimplemented!()
/// }
/// #[query(guard = "guard_function")]
/// fn query_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
///
/// To be able to make inter-canister calls from a query call, it must be a *composite* query (which cannot be executed in replicated mode).
///
/// ```rust
/// # use ic_cdk::query;
/// # fn wallet_canister_principal() -> candid::Principal { unimplemented!() }
/// #[query(composite = true)]
/// async fn composite_query_function() {
///    let (wallet_name,): (Option<String>,) = ic_cdk::call(wallet_canister_principal(), "name", ()).await.unwrap();
/// }
/// ```
///
/// If you would rather call the [`reply()`](crate::api::call::reply) function than return a value,
/// you will need to set `manual_reply` to `true` so that the canister does not trap.
///
/// ```rust
/// # fn calculate_result() {}
/// # type MyResult = ();
/// # use ic_cdk::query;
/// use ic_cdk::api::call::{self, ManualReply};
/// #[query(manual_reply = true)]
/// fn query_function() -> ManualReply<MyResult> {
///     let result = calculate_result();
///     ManualReply::one(result)
/// }
/// ```
pub use ic_cdk_macros::query;

/// Register an update call entry point.
///
/// This attribute macro will export a function with name `canister_update <name>`
/// in the canister module.
///
/// # Example
///
/// ```rust
/// # use ic_cdk::update;
/// #[update]
/// fn update_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
///
/// You can also specify the name of the exported function.
///
/// ```rust
/// # use ic_cdk::update;
/// #[update(name = "some_name")]
/// fn update_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
///
/// If you want to hide this method in the Candid generated by [export_candid!],
/// you will need to set `hidden` to `true`. The entry point still exists in the canister.
///
/// ```rust
/// # use ic_cdk::update;
/// #[update(hidden = true)]
/// fn update_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
///
/// You can specify a guard function to be executed before the update function.
/// When the guard function returns an error, the update function will not proceed.
///
/// ```rust
/// # use ic_cdk::update;
/// fn guard_function() -> Result<(), String> {
///     // ...
/// # unimplemented!()
/// }
/// #[update(guard = "guard_function")]
/// fn update_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
///
/// If you would rather call the [`reply()`](crate::api::call::reply) function than return a value,
/// you will need to set `manual_reply` to `true` so that the canister does not trap.
///
/// ```rust
/// # fn calculate_result() {}
/// # type MyResult = ();
/// # use ic_cdk::update;
/// use ic_cdk::api::call::{self, ManualReply};
/// #[update(manual_reply = true)]
/// fn update_function() -> ManualReply<MyResult> {
///     let result = calculate_result();
///     ManualReply::one(result)
/// }
/// ```
pub use ic_cdk_macros::update;

/// Register the `canister_init` entry point of a canister.
///
/// This attribute macro will export the function `canister_init`
/// in the canister module.
///
/// The function under this attribute must have no return value.
///
/// Each canister can only have one `canister_init` entry point.
///
/// # Example
///
/// ```rust
/// # use ic_cdk::init;
/// #[init]
/// fn init_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
///
/// The init function may accept an argument.
///
/// The argument must implement the `CandidType` trait.
///
/// And it should match the initialization parameters of the service constructor in the Candid interface.
///
/// ```rust
/// # use ic_cdk::init;
/// # use candid::*;
/// #[derive(Clone, Debug, CandidType, Deserialize)]
/// struct InitArg {
///     foo: u8,
/// }
///
/// #[init]
/// fn init_function(arg: InitArg) {
///     // ...
/// # unimplemented!()
/// }
/// ```
///
/// In this case, the argument will be read from `ic0.msg_arg_data_size/copy` and passed to the
/// init function upon successful deserialization.
///
///
/// Refer to the [`canister_init` Specification](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-init) for more information.
pub use ic_cdk_macros::init;

/// Register the `canister_pre_upgrade` entry point of a canister.
///
/// This attribute macro will export the function `canister_pre_upgrade`
/// in the canister module.
///
/// The function under this attribute must have no return value.
///
/// Each canister can only have one `canister_pre_upgrade` entry point.
///
/// # Example
///
/// ```rust
/// # use ic_cdk::pre_upgrade;
/// #[pre_upgrade]
/// fn pre_upgrade_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
pub use ic_cdk_macros::pre_upgrade;

/// Register the `canister_post_upgrade` entry point of a canister.
///
/// This attribute macro will export the function `canister_post_upgrade`
/// in the canister module.
///
/// The function under this attribute must have no return value.
///
/// Each canister can only have one `canister_post_upgrade` entry point.
///
/// # Example
///
/// ```rust
/// # use ic_cdk::post_upgrade;
/// #[post_upgrade]
/// fn post_upgrade_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
///
/// The post_upgrade function may accept an argument.
///
/// The argument must implement the `CandidType` trait.
///
/// And it should match the initialization parameters of the service constructor in the Candid interface.
/// Therefore, the init function and the post_upgrade function should take the same argument type.
///
/// ```rust
/// # use ic_cdk::post_upgrade;
/// # use candid::*;
/// #[derive(Clone, Debug, CandidType, Deserialize)]
/// struct InitArg {
///     foo: u8,
/// }
///
/// #[post_upgrade]
/// fn post_upgrade_function(arg: InitArg) {
///     // ...
/// # unimplemented!()
/// }
/// ```
///
/// In this case, the argument will be read from `ic0.msg_arg_data_size/copy` and passed to the
/// post_upgrade function upon successful deserialization.
pub use ic_cdk_macros::post_upgrade;

/// Register the `canister_heartbeat` entry point of a canister.
///
/// This attribute macro will export the function `canister_heartbeat`
/// in the canister module.
///
/// The function under this attribute must have no return value.
///
/// Each canister can only have one `canister_heartbeat` entry point.
///
/// # Example
///
/// ```rust
/// # use ic_cdk::heartbeat;
/// #[heartbeat]
/// fn heartbeat_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
pub use ic_cdk_macros::heartbeat;

/// Register the `canister_inspect_message` entry point of a canister.
///
/// This attribute macro will export the function `canister_inspect_message`
/// in the canister module.
///
/// The function under this attribute must have no return value.
///
/// Each canister can only have one `canister_inspect_message` entry point.
///
/// # Example
///
/// ```rust
/// # use ic_cdk::inspect_message;
/// #[inspect_message]
/// fn inspect_message_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
pub use ic_cdk_macros::inspect_message;

/// Register the `canister_on_low_wasm_memory` entry point of a canister.
///
/// This attribute macro will export the function `canister_on_low_wasm_memory`
/// in the canister module.
///
/// The function under this attribute must have no return value.
///
/// Each canister can only have one `canister_on_low_wasm_memory` entry point.
///
/// # Example
///
/// ```rust
/// # use ic_cdk::on_low_wasm_memory;
/// #[on_low_wasm_memory]
/// fn low_memory_handler() {
///     // ...
/// # unimplemented!()
/// }
/// ```
pub use ic_cdk_macros::on_low_wasm_memory;

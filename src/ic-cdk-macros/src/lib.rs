//! This crate provide a set of attribute macros to facilitate canister development.
//!
//! The macros fall into two categories:
//! * To register functions as canister entry points
//! * To export candid definitions
//!
//! ## Register functions as canister entry points
//!
//! These macros are directly related to the [Internet Computer Specification](https://smartcontracts.org/docs/interface-spec/index.html#_entry_points).
//!
//! * [`init`](attr.init.html)
//! * [`pre_upgrade`](attr.pre_upgrade.html)
//! * [`post_upgrade`](attr.post_upgrade.html)
//! * [`inspect_message`](attr.inspect_message.html)
//! * [`heartbeat`](attr.heartbeat.html)
//! * [`update`](attr.update.html)
//! * [`query`](attr.query.html)
//!
//! ## Export candid definitions
//!
//! * [`export_candid`](attr.export_candid.html)

#![warn(
    elided_lifetimes_in_paths,
    missing_debug_implementations,
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc
)]

use proc_macro::TokenStream;
use std::sync::atomic::{AtomicU32, Ordering};
use syn::Error;

mod export;

// To generate unique identifiers for functions and arguments
static NEXT_ID: AtomicU32 = AtomicU32::new(0);
pub(crate) fn id() -> u32 {
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

fn handle_debug_and_errors<F>(
    cb: F,
    name: &str,
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream
where
    F: FnOnce(
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
    ) -> Result<proc_macro2::TokenStream, Error>,
{
    if std::env::var_os("IC_CDK_DEBUG").is_some() {
        eprintln!("--- IC_CDK_MACROS DEBUG ---");
        eprintln!("{}\n    attr: {}\n    item: {}", name, attr, item);
    }

    let result = cb(attr.into(), item.into());

    if std::env::var_os("IC_CDK_DEBUG").is_some() {
        eprintln!("--------- RESULT  ---------");
        if let Ok(ref stream) = result {
            eprintln!("{}", stream);
        }
        eprintln!("---------------------------");
    }

    result.map_or_else(|e| e.to_compile_error().into(), Into::into)
}

/// Create a WASI start function which print the Candid interface of the canister.
///
/// Requiring "wasi" feature enabled. Or the function will have empty body.
///
/// Call this macro only if you want the Candid export behavior.
/// Only call it once at the end of canister code outside query/update definition.
#[cfg(feature = "export_candid")]
#[proc_macro]
pub fn export_candid(input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    quote::quote! {
        ::candid::export_service!(#input);

        #[no_mangle]
        pub unsafe extern "C" fn _start() {
            let result = __export_service();
            let ret = unsafe { ::ic_cdk::api::wasi::print(&result) };
            ::ic_cdk::api::wasi::proc_exit(ret as u32);
        }
    }
    .into()
}

#[doc(hidden)]
#[cfg(not(feature = "export_candid"))]
#[proc_macro]
pub fn export_candid(_: TokenStream) -> TokenStream {
    quote::quote! {}.into()
}

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
/// If you would rather call the [`call::reply`] function than return a value,
/// you will need to set `manual_reply` to `true` so that the canister does not
/// trap.
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
///
/// [`call::reply`]: https://docs.rs/ic-cdk/latest/ic_cdk/api/call/fn.reply.html
#[proc_macro_attribute]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_query, "ic_query", attr, item)
}

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
/// If you would rather call the [`call::reply`] function than return a value,
/// you will need to set `manual_reply` to `true` so that the canister does not
/// trap.
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
///
/// [`call::reply`]: https://docs.rs/ic-cdk/latest/ic_cdk/api/call/fn.reply.html
#[proc_macro_attribute]
pub fn update(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_update, "ic_update", attr, item)
}

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
/// The init function may accept an argument, if that argument is a `CandidType`:
///
/// ```rust
/// # use ic_cdk::init;
/// # use candid::*;
///
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
/// Refer to the [`canister_init` Specification](https://smartcontracts.org/docs/interface-spec/index.html#system-api-init) for more information.
#[proc_macro_attribute]
pub fn init(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_init, "ic_init", attr, item)
}

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
#[proc_macro_attribute]
pub fn pre_upgrade(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_pre_upgrade, "ic_pre_upgrade", attr, item)
}

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
#[proc_macro_attribute]
pub fn post_upgrade(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_post_upgrade, "ic_post_upgrade", attr, item)
}

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
#[proc_macro_attribute]
pub fn heartbeat(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_heartbeat, "ic_heartbeat", attr, item)
}

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
#[proc_macro_attribute]
pub fn inspect_message(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_inspect_message, "ic_inspect_message", attr, item)
}

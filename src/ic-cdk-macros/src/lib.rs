//! This crate provide a set of attribute macros to faciliate canister development.
//!
//! The macros fall into two categories:
//! * To register functions as canister entry points
//! * To import another canister as a rust struct for inter-canister operation.
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
//! ## Import another canister as a Rust module
//!
//! * [`import`](attr.import.html)

use proc_macro::TokenStream;
use std::sync::atomic::{AtomicU32, Ordering};
use syn::Error;

mod export;
mod import;

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

/// Register a query call entry point.
///
/// This attribute macro will export a function with name `canister_query <name>`
/// in the canister module.
///
/// # Example
///
/// ```rust
/// # use ic_cdk_macros::query;
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
/// # use ic_cdk_macros::*;
/// #[query(name = "some_name")]
/// fn query_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
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
/// # use ic_cdk_macros::update;
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
/// # use ic_cdk_macros::*;
/// #[update(name = "some_name")]
/// fn update_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
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
/// # use ic_cdk_macros::init;
/// #[init]
/// fn init_function() {
///     // ...
/// # unimplemented!()
/// }
/// ```
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
/// # use ic_cdk_macros::pre_upgrade;
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
/// # use ic_cdk_macros::post_upgrade;
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
/// # use ic_cdk_macros::heartbeat;
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
/// # use ic_cdk_macros::inspect_message;
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

/// Import another canister as a Rust module.
///
/// The attribute can be invoked two ways: `import(canister = "foo")`, which will automatically import it from a compatible canister
/// governed by the same DFX instance after you run `dfx build`, and `import(canister_id = "rrkah-fqaaa-aaaaa-aaaaq-cai", candid_path = "foo.did")`,
/// which does it the boring way.
///
/// All public interfaces defined in the corresponding candid file can be accessed through the annotated module. Both functions and types will be exported,
/// but in the case that names collide, functions will 'win' and types can be accessed through a submodule named `t`.
///
/// Inline variants and records cannot be directly translated, so they have names that look like `recordname_field0_variant3_param2`. These names are
/// deterministic, and should not change when the Candid file is changed backwards-compatibly.
///
/// Every symbol can be overridden by writing your preferred version of it in the attributed module - structs, enums, functions, etc.
/// See [`Processor::new`](ic_cdk_codegen::Processor::new) for the rules on doing so. You can even override the base Candid primitives,
/// for example to use `u128` instead of [`Nat`](candid::Nat) (the names for doing so follow [`PrimType`](candid::parser::types::PrimType)
/// rather than the IDL notation).
///
/// # Example
///
/// You can specify the local canister with its name. Note that this approach relies on the project organization by [dfx](https://github.com/dfinity/sdk).
/// During `dfx build`, the imported canister will be correctly resolved.
///
/// ```rust,ignore
/// # use ic_cdk_macros::import;
/// #[import(canister = "some_canister")]
/// mod some_canister {}
/// ```
///
/// Or you can specify both the `canister_id` and the `candid_path`. This example also shows how you might override some declarations.
///
/// ```rust,ignore
/// # use ic_cdk_macros::import;
/// #[import(canister_id = "aaaaa-aa", candid_path = "path/to/ic.did")]
/// mod ic {
///     pub type Nat = u128;
///     pub struct CanisterIdContainer {
///         canister_id: Principal,
///     }
///     pub async fn create_canister(can: CanisterIdContainer) -> CanisterIdContainer;
/// }
/// ```
///
/// # Note
///
/// If your editor does not support expanding proc macros, the [`ic_cdk_codegen`] crate can also be used directly in a build script.
#[proc_macro_attribute]
pub fn import(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(import::ic_import, "ic_import", attr, item)
}

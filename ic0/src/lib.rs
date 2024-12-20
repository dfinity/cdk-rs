//! Raw bindings to the [Internet Computer system API](https://internetcomputer.org/docs/current/references/ic-interface-spec#system-api-imports).

#![warn(
    elided_lifetimes_in_paths,
    missing_debug_implementations,
    unsafe_op_in_unsafe_fn,
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc
)]

mod ic0;
pub use crate::ic0::*;

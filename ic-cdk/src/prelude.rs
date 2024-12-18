//! The prelude module contains the most commonly used types and traits.
pub use crate::api::{msg_caller, id, debug_print, trap};
pub use crate::call::{Call, CallResult, ConfigurableCall, RejectCode, SendableCall};
pub use crate::macros::{
    export_candid, heartbeat, init, inspect_message, post_upgrade, pre_upgrade, query, update,
};
pub use crate::{eprintln, println, setup, spawn};

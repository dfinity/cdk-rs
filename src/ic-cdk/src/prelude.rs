//! The prelude module contains the most commonly used types and traits.
pub use crate::api::call::{Call, CallResult, ConfigurableCall, RejectionCode, SendableCall};
pub use crate::api::{caller, id, print, trap};
pub use crate::macros::{
    export_candid, heartbeat, init, inspect_message, post_upgrade, pre_upgrade, query, update,
};
pub use crate::{eprintln, println, setup, spawn};

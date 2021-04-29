#![allow(clippy::all)]
#![allow(dead_code)]

//! This files represent the API endpoints for the IC System API.
//! It is meant to be a copy-paste of the System API from the spec,
//! and also not exported outside this crate.
//!
//! Each of these functions are in a private module accessible only
//! in this crate. Each function should have a rust-typed version here
//! as an export point, and have a fully counterpart that is public
//! and declared in [api.rs].
//!
//! An example is arg data; the msg_arg_data_copy() takes a pointer
//! and a length, there should be two versions of this API endpoint:
//!
//! 1. [ic0::private::msg_arg_data_copy(i32, i32) -> ()] that is the
//!    actual export of the system api.
//! 2. [api::msg_arg_data() -> Vec<u8>] which calls the size, allocate
//!    a buffer, and fills it with the data itself.

// These two macros are used to being able to copy-paste the system API imports from the
// spec without actually changing anything. This makes it possible to generate at build
// time the list of imports from the spec. We don't do that (yet) as the spec isn't
// open sourced.
// The exported methods are in an `internal` module.
macro_rules! _ic0_module_ret {
    ( ( $_: ident : $t: ty ) ) => {
        $t
    };
    ( ( $t: ty ) ) => {
        $t
    };
    ( $t: ty ) => {
        $t
    };
}

// Declare the module itself as a list of API endpoints.
macro_rules! ic0_module {
    ( $( ic0. $name: ident : ( $( $argname: ident : $argtype: ty ),* ) -> $rettype: tt ; )+ ) => {

        #[cfg(target_arch = "wasm32")]
        #[link(wasm_import_module = "ic0")]
        extern "C" {
            $(pub(super) fn $name($( $argname: $argtype, )*) -> _ic0_module_ret!($rettype) ;)*
        }

        $(
        #[cfg(not(target_arch = "wasm32"))]
        pub(super) unsafe fn $name($( $argname: $argtype, )*) -> _ic0_module_ret!($rettype) {
            let _ = ( $( $argname, )* );  // make sure the arguments are used.
            panic!("{} should only be called inside canisters.", stringify!( $name ));
        }
        )*
    };
}

// This is a private module that can only be used internally in this file.
// Copy-paste the spec section of the API here.
ic0_module! {
    ic0.msg_arg_data_size : () -> i32;                                          // I U Q Ry
    ic0.msg_arg_data_copy : (dst : i32, offset : i32, size : i32) -> ();        // I U Q Ry
    ic0.msg_caller_size : () -> i32;                                            // I G U Q
    ic0.msg_caller_copy : (dst : i32, offset: i32, size : i32) -> ();           // I G U Q
    ic0.msg_reject_code : () -> i32;                                            // Ry Rt
    ic0.msg_reject_msg_size : () -> i32;                                        // Rt
    ic0.msg_reject_msg_copy : (dst : i32, offset : i32, size : i32) -> ();      // Rt

    ic0.msg_reply_data_append : (src : i32, size : i32) -> ();                  // U Q Ry Rt
    ic0.msg_reply : () -> ();                                                   // U Q Ry Rt
    ic0.msg_reject : (src : i32, size : i32) -> ();                             // U Q Ry Rt

    ic0.msg_cycles_available : () -> i64;                                       // U Rt Ry
    ic0.msg_cycles_refunded : () -> i64;                                        // Rt Ry
    ic0.msg_cycles_accept : ( max_amount : i64 ) -> ( amount : i64 );           // U Rt Ry

    ic0.canister_self_size : () -> i32;                                         // *
    ic0.canister_self_copy : (dst : i32, offset : i32, size : i32) -> ();       // *
    ic0.canister_cycle_balance : () -> i64;                                     // *
    ic0.canister_status : () -> i32;                                            // *

    ic0.call_new :                                                              // U Ry Rt
    ( callee_src  : i32,
      callee_size : i32,
      name_src : i32,
      name_size : i32,
      reply_fun : i32,
      reply_env : i32,
      reject_fun : i32,
      reject_env : i32
    ) -> ();
    ic0.call_data_append : (src : i32, size : i32) -> ();                       // U Ry Rt
    ic0.call_cycles_add : ( amount : i64 ) -> ();                               // U Ry Rt
    ic0.call_perform : () -> ( err_code : i32 );                                // U Ry Rt

    ic0.stable_size : () -> (page_count : i32);                                 // *
    ic0.stable_grow : (new_pages : i32) -> (old_page_count : i32);              // *
    ic0.stable_write : (offset : i32, src : i32, size : i32) -> ();             // *
    ic0.stable_read : (dst : i32, offset : i32, size : i32) -> ();              // *

    ic0.certified_data_set : (src: i32, size: i32) -> ();                        // I G U Ry Rt
    ic0.data_certificate_present : () -> i32;                                    // Q
    ic0.data_certificate_size : () -> i32;                                       // Q
    ic0.data_certificate_copy : (dst: i32, offset: i32, size: i32) -> ();        // Q

    ic0.time : () -> (timestamp : i64);                                         // *

    ic0.debug_print : (src : i32, size : i32) -> ();                            // * s
    ic0.trap : (src : i32, size : i32) -> ();                                   // * s
}

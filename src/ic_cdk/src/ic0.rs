#![allow(dead_code)]

//! This files represent the API endpoints for the IC System API.
//! It is meant to be a copy-paste of the System API from the spec,
//! and also not exported outside this crate.
//!
//! Each of these functions should have a counterpart that is
//! public and declared in [api.rs].

// These two macros are used to being able to copy-paste the system API imports from the
// spec without actually changing anything. This makes it possible to generate at build
// time the list of imports from the spec. We don't do that (yet) as the spec isn't
// open sourced.
// The exported methods are in an `internal` module.
macro_rules! _ic0_module_ret {
    ( ( $_: ident : $t: ty ) ) => {
        $t
    };
    ( $t: ty ) => {
        $t
    };
}

macro_rules! ic0_module {
    ( $( ic0. $name: ident : ( $( $argname: ident : $argtype: ty ),* ) -> $rettype: tt ; )+ ) => {

        #[cfg(target_arch = "wasm32")]
        #[link(wasm_import_module = "ic0")]
        extern "C" {
          $(pub(crate) fn $name($( $argname: $argtype, )*) -> _ic0_module_ret!($rettype) ;)*
        }

        $(
        #[cfg(not(target_arch = "wasm32"))]
        pub(crate) unsafe fn $name($( $argname: $argtype, )*) -> _ic0_module_ret!($rettype) {
                            let _ = ( $( $argname, )* );
            panic!("{} should only be called inside canisters.", stringify!( $name ));
        }
        )*
    };
}

// Copy-paste the spec section of the API here.
ic0_module! {
    ic0.msg_arg_data_size : () -> i32;                                      // I U Q Ry
    ic0.msg_arg_data_copy : (dst : i32, offset : i32, size : i32) -> ();    // I U Q Ry
    // ic0.msg_method_size : () -> i32;                                        // P
    // ic0.msg_method_copy : (dst : i32, offset: i32, size : i32) -> ();       // P
    ic0.msg_caller_size : () -> i32;                                        // I P G U Q
    ic0.msg_caller_copy : (dst : i32, offset: i32, size : i32) -> ();       // I P G U Q
    ic0.msg_reject_code : () -> i32;                                        // Ry Rt
    ic0.msg_reject_msg_size : () -> i32;                                    // Rt
    ic0.msg_reject_msg_copy : (dst : i32, offset : i32, size : i32) -> ();  // Rt
    ic0.msg_reply_data_append : (src : i32, size : i32) -> ();              // U Q Ry Rt
    ic0.msg_reply : () -> ();                                               // U Q Ry Rt
    ic0.msg_reject : (src : i32, size : i32) -> ();                         // U Q Ry Rt
    // ic0.pay_for_ingress :                                                   // P
    //   ( bucket_src : i32,
    //     bucket_size : i32,
    //     cost : i64,
    //     limit : i64
    //   ) -> ();
    ic0.canister_self_size : () -> i32;                                     // *
    ic0.canister_self_copy : (dst : i32, offset : i32, size : i32) -> ();   // *
    ic0.call_simple :                                                       // U Ry Rt
      ( callee_src  : i32,
        callee_size : i32,
        name_src    : i32,
        name_size   : i32,
        reply_fun   : i32,
        reply_env   : i32,
        reject_fun  : i32,
        reject_env  : i32,
        data_src    : i32,
        data_size   : i32
      ) -> ( err_code : i32 );
    // ic0.stable_size : () -> (page_count : i32);                             // *
    // ic0.stable_grow : (new_pages : i32) -> (old_page_count : i32);          // *
    // ic0.stable_write : (offset : i32, src : i32, size : i32) -> ();         // *
    // ic0.stable_read : (dst : i32, offset : i32, size : i32) -> ();          // *
    ic0.time : () -> (timestamp : i64);                                     // *
    ic0.debug_print : (src : i32, size : i32) -> ();                        // * s
    ic0.trap : (src : i32, size : i32) -> ();                               // * s

    // Cycle management.
    ic0.canister_gas_count: () -> i64;
    ic0.msg_received_gas: () -> i64;
}

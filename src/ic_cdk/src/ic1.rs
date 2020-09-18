#![cfg(feature = "experimental")]

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
macro_rules! _ic1_module_ret {
    ( ( $_: ident : $t: ty ) ) => {
        $t
    };
    ( $t: ty ) => {
        $t
    };
}

macro_rules! ic1_module {
    ( $( ic1. $name: ident : ( $( $argname: ident : $argtype: ty ),* ) -> $rettype: tt ; )+ ) => {

        #[cfg(target_arch = "wasm32")]
        #[link(wasm_import_module = "ic1")]
        extern "C" {
          $(pub(crate) fn $name($( $argname: $argtype, )*) -> _ic1_module_ret!($rettype) ;)*
        }

        $(
        #[cfg(not(target_arch = "wasm32"))]
        pub(crate) unsafe fn $name($( $argname: $argtype, )*) -> _ic1_module_ret!($rettype) {
                            let _ = ( $( $argname, )* );
            panic!("{} should only be called inside canisters.", stringify!( $name ));
        }
        )*
    };
}

// Copy-paste the spec section of the API here.
ic1_module! {
    ic1.msg_reply: (gas_to_keep: i64) -> ();
    ic1.call_simple: (
        callee_src: i32,
        callee_size: i32,
        name_src: i32,
        name_size: i32,
        reply_fun: i32,
        reply_env: i32,
        reject_fun: i32,
        reject_env: i32,
        data_src: i32,
        data_size: i32,
        gas: i64
    ) -> i32;
}

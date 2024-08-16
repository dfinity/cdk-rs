//! This module is only for internal usage of [crate::export_candid] macro.
//!
//! A simple WASI binding. It's only enabled by the "wasi" feature to allow building
//! the canister as a standalone WASI binary that can output its Candid interface in wasmtime.
//!
//! NEVER use them in Canister code.

type Fd = u32;
type Size = usize;
pub type Errno = i32;
pub type Rval = u32;

#[derive(Debug)]
#[repr(C)]
pub struct Ciovec {
    buf: *const u8,
    buf_len: Size,
}

#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    pub fn fd_write(fd: Fd, iovs_ptr: *const Ciovec, iovs_len: Size, nwritten: *mut Size) -> Errno;
    pub fn proc_exit(rval: Rval);
}
pub unsafe fn print(text: &str) -> Errno {
    let ciovec = Ciovec {
        buf: text.as_ptr(),
        buf_len: text.len(),
    };
    let ciovecs = [ciovec];
    let mut nwritten = 0;
    unsafe { fd_write(1, ciovecs.as_ptr(), ciovecs.len(), &mut nwritten) }
}

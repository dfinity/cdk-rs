type Fd = u32;
type Size = usize;
pub type Errno = i32;
pub type Rval = u32;

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
    fd_write(1, ciovecs.as_ptr(), ciovecs.len(), &mut nwritten)
}

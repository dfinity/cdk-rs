use super::*;

/// A standard implementation of [`StableMemory`].
///
/// Useful for creating [`StableWriter`] and [`StableReader`].
#[derive(Default, Debug, Copy, Clone)]
pub struct CanisterStableMemory {}

impl StableMemory for CanisterStableMemory {
    fn stable_size(&self) -> u64 {
        // SAFETY: ic0.stable64_size is always safe to call.
        unsafe { ic0::stable64_size() }
    }

    fn stable_grow(&self, new_pages: u64) -> Result<u64, StableMemoryError> {
        // SAFETY: ic0.stable64_grow is always safe to call.
        unsafe {
            match ic0::stable64_grow(new_pages) {
                u64::MAX => Err(StableMemoryError::OutOfMemory),
                x => Ok(x),
            }
        }
    }

    fn stable_write(&self, offset: u64, buf: &[u8]) {
        // SAFETY: `buf`, being &[u8], is a readable sequence of bytes, and therefore valid to pass to ic0.stable64_write.
        unsafe {
            ic0::stable64_write(offset, buf.as_ptr() as u64, buf.len() as u64);
        }
    }

    fn stable_read(&self, offset: u64, buf: &mut [u8]) {
        // SAFETY: `buf`, being &mut [u8], is a writable sequence of bytes, and therefore valid to pass to ic0.stable64_read.
        unsafe {
            ic0::stable64_read(buf.as_ptr() as u64, offset, buf.len() as u64);
        }
    }
}

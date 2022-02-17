//! APIs to manage stable memory.
//!
//! You can check the [Internet Computer Specification](https://smartcontracts.org/docs/interface-spec/index.html#system-api-stable-memory)
//! for a in-depth explanation of stable memory.
use std::{error, fmt, io};

/// Gets current size of the stable memory.
pub fn stable_size() -> u32 {
    unsafe { super::ic0::stable_size() as u32 }
}

/// Similar to `stable_size` but with support for 64-bit addressed memory.
pub fn stable64_size() -> u64 {
    unsafe { super::ic0::stable64_size() as u64 }
}

/// A possible error value when dealing with stable memory.
#[derive(Debug)]
pub enum StableMemoryError {
    /// No more stable memory could be allocated.
    OutOfMemory,
    /// Attempted to read more stable memory than had been allocated.
    OutOfBounds,
}

impl fmt::Display for StableMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::OutOfMemory => f.write_str("Out of memory"),
            Self::OutOfBounds => f.write_str("Read exceeds allocated memory"),
        }
    }
}

impl error::Error for StableMemoryError {}

/// Attempts to grow the stable memory by `new_pages` (added pages).
///
/// Returns an error if it wasn't possible. Otherwise, returns the previous
/// size that was reserved.
///
/// *Note*: Pages are 64KiB in WASM.
pub fn stable_grow(new_pages: u32) -> Result<u32, StableMemoryError> {
    unsafe {
        match super::ic0::stable_grow(new_pages as i32) {
            -1 => Err(StableMemoryError::OutOfMemory),
            x => Ok(x as u32),
        }
    }
}

/// Similar to `stable_grow` but with support for 64-bit addressed memory.
pub fn stable64_grow(new_pages: u64) -> Result<u64, StableMemoryError> {
    unsafe {
        match super::ic0::stable64_grow(new_pages as i64) {
            -1 => Err(StableMemoryError::OutOfMemory),
            x => Ok(x as u64),
        }
    }
}

/// Writes data to the stable memory location specified by an offset.
pub fn stable_write(offset: u32, buf: &[u8]) {
    unsafe {
        super::ic0::stable_write(offset as i32, buf.as_ptr() as i32, buf.len() as i32);
    }
}

/// Similar to `stable_write` but with support for 64-bit addressed memory.
pub fn stable64_write(offset: u64, buf: &[u8]) {
    unsafe {
        super::ic0::stable64_write(offset as i64, buf.as_ptr() as i64, buf.len() as i64);
    }
}

/// Reads data from the stable memory location specified by an offset.
pub fn stable_read(offset: u32, buf: &mut [u8]) {
    unsafe {
        super::ic0::stable_read(buf.as_ptr() as i32, offset as i32, buf.len() as i32);
    }
}

/// Similar to `stable_read` but with support for 64-bit addressed memory.
pub fn stable64_read(offset: u64, buf: &mut [u8]) {
    unsafe {
        super::ic0::stable64_read(buf.as_ptr() as i64, offset as i64, buf.len() as i64);
    }
}

/// Returns a copy of the stable memory.
///
/// This will map the whole memory (even if not all of it has been written to).
pub fn stable_bytes() -> Vec<u8> {
    let size = (stable_size() as usize) << 16;
    let mut vec = Vec::with_capacity(size);
    unsafe {
        vec.set_len(size);
    }

    stable_read(0, vec.as_mut_slice());

    vec
}

/// A writer to the stable memory.
///
/// Will attempt to grow the memory as it writes,
/// and keep offsets and total capacity.
pub struct StableWriter {
    /// The offset of the next write.
    offset: usize,

    /// The capacity, in pages.
    capacity: u32,
}

impl Default for StableWriter {
    fn default() -> Self {
        let capacity = stable_size();

        Self {
            offset: 0,
            capacity,
        }
    }
}

impl StableWriter {
    /// Attempts to grow the memory by adding new pages.
    pub fn grow(&mut self, added_pages: u32) -> Result<(), StableMemoryError> {
        let old_page_count = stable_grow(added_pages)?;
        self.capacity = old_page_count + added_pages;
        Ok(())
    }

    /// Writes a byte slice to the buffer.
    ///
    /// The only condition where this will
    /// error out is if it cannot grow the memory.
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, StableMemoryError> {
        if self.offset + buf.len() > ((self.capacity as usize) << 16) {
            self.grow((buf.len() >> 16) as u32 + 1)?;
        }

        stable_write(self.offset as u32, buf);
        self.offset += buf.len();
        Ok(buf.len())
    }
}

impl io::Write for StableWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.write(buf)
            .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        // Noop.
        Ok(())
    }
}

/// A reader to the stable memory.
///
/// Keeps an offset and reads off stable memory consecutively.
pub struct StableReader {
    /// The offset of the next read.
    offset: usize,
    /// The capacity, in pages.
    capacity: u32,
}

impl Default for StableReader {
    fn default() -> Self {
        Self {
            offset: 0,
            capacity: stable_size(),
        }
    }
}

impl StableReader {
    /// Reads data from the stable memory location specified by an offset.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, StableMemoryError> {
        let cap = (self.capacity as usize) << 16;
        let read_buf = if buf.len() + self.offset > cap {
            if self.offset < cap {
                &mut buf[..cap - self.offset]
            } else {
                return Err(StableMemoryError::OutOfBounds);
            }
        } else {
            buf
        };
        stable_read(self.offset as u32, read_buf);
        self.offset += read_buf.len();
        Ok(read_buf.len())
    }
}

impl io::Read for StableReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.read(buf).or(Ok(0)) // Read defines EOF to be success
    }
}

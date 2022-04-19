//! APIs to manage stable memory.
//!
//! You can check the [Internet Computer Specification](https://smartcontracts.org/docs/interface-spec/index.html#system-api-stable-memory)
//! for a in-depth explanation of stable memory.
mod canister;
#[cfg(test)]
mod tests;

use canister::CanisterStableMemory;
use std::{error, fmt, io};

const WASM_PAGE_SIZE_IN_BYTES: usize = 64 * 1024; // 64KB

static CANISTER_STABLE_MEMORY: CanisterStableMemory = CanisterStableMemory {};

/// A trait defining the stable memory API which each canister running on the IC can make use of
pub trait StableMemory {
    /// Gets current size of the stable memory (in WASM pages).
    fn stable_size(&self) -> u32;

    /// Similar to `stable_size` but with support for 64-bit addressed memory.
    fn stable64_size(&self) -> u64;

    /// Attempts to grow the stable memory by `new_pages` (added pages).
    ///
    /// Returns an error if it wasn't possible. Otherwise, returns the previous
    /// size that was reserved.
    ///
    /// *Note*: Pages are 64KiB in WASM.
    fn stable_grow(&self, new_pages: u32) -> Result<u32, StableMemoryError>;

    /// Similar to `stable_grow` but with support for 64-bit addressed memory.
    fn stable64_grow(&self, new_pages: u64) -> Result<u64, StableMemoryError>;

    /// Writes data to the stable memory location specified by an offset.
    ///
    /// Warning - this will panic if `offset + buf.len()` exceeds the current size of stable memory.
    /// Use `stable_grow` to request more stable memory if needed.
    fn stable_write(&self, offset: u32, buf: &[u8]);

    /// Similar to `stable_write` but with support for 64-bit addressed memory.
    fn stable64_write(&self, offset: u64, buf: &[u8]);

    /// Reads data from the stable memory location specified by an offset.
    fn stable_read(&self, offset: u32, buf: &mut [u8]);

    /// Similar to `stable_read` but with support for 64-bit addressed memory.
    fn stable64_read(&self, offset: u64, buf: &mut [u8]);
}

/// Gets current size of the stable memory (in WASM pages).
pub fn stable_size() -> u32 {
    CANISTER_STABLE_MEMORY.stable_size()
}

/// Similar to `stable_size` but with support for 64-bit addressed memory.
pub fn stable64_size() -> u64 {
    CANISTER_STABLE_MEMORY.stable64_size()
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
    CANISTER_STABLE_MEMORY.stable_grow(new_pages)
}

/// Similar to `stable_grow` but with support for 64-bit addressed memory.
pub fn stable64_grow(new_pages: u64) -> Result<u64, StableMemoryError> {
    CANISTER_STABLE_MEMORY.stable64_grow(new_pages)
}

/// Writes data to the stable memory location specified by an offset.
///
/// Warning - this will panic if `offset + buf.len()` exceeds the current size of stable memory.
/// Use `stable_grow` to request more stable memory if needed.
pub fn stable_write(offset: u32, buf: &[u8]) {
    CANISTER_STABLE_MEMORY.stable_write(offset, buf)
}

/// Similar to `stable_write` but with support for 64-bit addressed memory.
pub fn stable64_write(offset: u64, buf: &[u8]) {
    CANISTER_STABLE_MEMORY.stable64_write(offset, buf)
}

/// Reads data from the stable memory location specified by an offset.
pub fn stable_read(offset: u32, buf: &mut [u8]) {
    CANISTER_STABLE_MEMORY.stable_read(offset, buf)
}

/// Similar to `stable_read` but with support for 64-bit addressed memory.
pub fn stable64_read(offset: u64, buf: &mut [u8]) {
    CANISTER_STABLE_MEMORY.stable64_read(offset, buf)
}

/// Returns a copy of the stable memory.
///
/// This will map the whole memory (even if not all of it has been written to).
pub fn stable_bytes() -> Vec<u8> {
    let size = (stable_size() as usize) << 16;
    let mut vec = Vec::with_capacity(size);
    unsafe {
        super::ic0::stable_read(vec.as_ptr() as i32, 0, size as i32);
        vec.set_len(size);
    }
    vec
}

/// A writer to the stable memory.
///
/// Warning: This will overwrite any existing data in stable memory as it writes, so ensure you set
/// the `offset` value accordingly if you wish to preserve existing data.
///
/// Will attempt to grow the memory as it writes,
/// and keep offsets and total capacity.
pub struct StableWriter<M: StableMemory = CanisterStableMemory> {
    /// The offset of the next write.
    offset: usize,

    /// The capacity, in pages.
    capacity: u32,

    /// The stable memory to write data to.
    memory: M,
}

impl Default for StableWriter {
    fn default() -> Self {
        Self::with_memory(CanisterStableMemory::default(), 0)
    }
}

impl<M: StableMemory> StableWriter<M> {
    /// Creates a new `StableWriter` which writes to the selected memory
    pub fn with_memory(memory: M, offset: usize) -> Self {
        let capacity = memory.stable_size();

        Self {
            offset,
            capacity,
            memory,
        }
    }

    /// Attempts to grow the memory by adding new pages.
    pub fn grow(&mut self, new_pages: u32) -> Result<(), StableMemoryError> {
        let old_page_count = self.memory.stable_grow(new_pages)?;
        self.capacity = old_page_count + new_pages;
        Ok(())
    }

    /// Writes a byte slice to the buffer.
    ///
    /// The only condition where this will
    /// error out is if it cannot grow the memory.
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, StableMemoryError> {
        let required_capacity_bytes = self.offset + buf.len();
        let required_capacity_pages = ((required_capacity_bytes + WASM_PAGE_SIZE_IN_BYTES - 1)
            / WASM_PAGE_SIZE_IN_BYTES) as u32;
        let current_pages = self.capacity;
        let additional_pages_required = required_capacity_pages.saturating_sub(current_pages);

        if additional_pages_required > 0 {
            self.grow(additional_pages_required)?;
        }

        self.memory.stable_write(self.offset as u32, buf);
        self.offset += buf.len();
        Ok(buf.len())
    }
}

impl<M: StableMemory> io::Write for StableWriter<M> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.write(buf)
            .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        // Noop.
        Ok(())
    }
}

/// A writer to the stable memory which first writes the bytes to an in memory buffer and flushes
/// the buffer to stable memory each time it becomes full.
///
/// Warning: This will overwrite any existing data in stable memory as it writes, so ensure you set
/// the `offset` value accordingly if you wish to preserve existing data.
///
/// Note: Each call to grow or write to stable memory is a relatively expensive operation, so pick a
/// buffer size large enough to avoid excessive calls to stable memory.
pub struct BufferedStableWriter<M: StableMemory = CanisterStableMemory> {
    inner: io::BufWriter<StableWriter<M>>,
}

impl BufferedStableWriter {
    /// Creates a new `BufferedStableWriter`
    pub fn new(buffer_size: usize) -> BufferedStableWriter {
        BufferedStableWriter::with_writer(buffer_size, StableWriter::default())
    }
}

impl<M: StableMemory> BufferedStableWriter<M> {
    /// Creates a new `BufferedStableWriter` which writes to the selected memory
    pub fn with_writer(buffer_size: usize, writer: StableWriter<M>) -> BufferedStableWriter<M> {
        BufferedStableWriter {
            inner: io::BufWriter::with_capacity(buffer_size, writer),
        }
    }
}

impl<M: StableMemory> io::Write for BufferedStableWriter<M> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// A reader to the stable memory.
///
/// Keeps an offset and reads off stable memory consecutively.
pub struct StableReader<M: StableMemory = CanisterStableMemory> {
    /// The offset of the next read.
    offset: usize,

    /// The capacity, in pages.
    capacity: u32,

    /// The stable memory to read data from.
    memory: M,
}

impl Default for StableReader {
    fn default() -> Self {
        Self::with_memory(CanisterStableMemory::default(), 0)
    }
}

impl<M: StableMemory> StableReader<M> {
    /// Creates a new `StableReader` which reads from the selected memory
    pub fn with_memory(memory: M, offset: usize) -> Self {
        let capacity = memory.stable_size();

        Self {
            offset,
            capacity,
            memory,
        }
    }

    /// Reads data from the stable memory location specified by an offset.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, StableMemoryError> {
        let capacity_bytes = self.capacity as usize * WASM_PAGE_SIZE_IN_BYTES;
        let read_buf = if buf.len() + self.offset > capacity_bytes {
            if self.offset < capacity_bytes {
                &mut buf[..capacity_bytes - self.offset]
            } else {
                return Err(StableMemoryError::OutOfBounds);
            }
        } else {
            buf
        };
        self.memory.stable_read(self.offset as u32, read_buf);
        self.offset += read_buf.len();
        Ok(read_buf.len())
    }
}

impl<M: StableMemory> io::Read for StableReader<M> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.read(buf).or(Ok(0)) // Read defines EOF to be success
    }
}

/// A reader to the stable memory which reads bytes a chunk at a time as each chunk is required.
pub struct BufferedStableReader<M: StableMemory = CanisterStableMemory> {
    inner: io::BufReader<StableReader<M>>,
}

impl BufferedStableReader {
    /// Creates a new `BufferedStableReader`
    pub fn new(buffer_size: usize) -> BufferedStableReader {
        BufferedStableReader::with_reader(buffer_size, StableReader::default())
    }
}

impl<M: StableMemory> BufferedStableReader<M> {
    /// Creates a new `BufferedStableReader` which reads from the selected memory
    pub fn with_reader(buffer_size: usize, reader: StableReader<M>) -> BufferedStableReader<M> {
        BufferedStableReader {
            inner: io::BufReader::with_capacity(buffer_size, reader),
        }
    }
}

impl<M: StableMemory> io::Read for BufferedStableReader<M> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

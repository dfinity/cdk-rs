//! APIs to manage stable memory.
//!
//! You can check the [Internet Computer Specification](https://smartcontracts.org/docs/interface-spec/index.html#system-api-stable-memory)
//! for a in-depth explanation of stable memory.
mod canister;
#[cfg(test)]
mod tests;

use canister::CanisterStableMemory;
use std::cmp::Ordering;
use std::{error, fmt, io};

const WASM_PAGE_SIZE_IN_BYTES: u64 = 64 * 1024; // 64KB

trait StableMemory {
    fn stable_size(&self) -> u32;
    fn stable64_size(&self) -> u64;
    fn stable_grow(&self, new_pages: u32) -> Result<u32, StableMemoryError>;
    fn stable64_grow(&self, new_pages: u64) -> Result<u64, StableMemoryError>;
    fn stable_write(&self, offset: u32, buf: &[u8]);
    fn stable64_write(&self, offset: u64, buf: &[u8]);
    fn stable_read(&self, offset: u32, buf: &mut [u8]);
    fn stable64_read(&self, offset: u64, buf: &mut [u8]);
}

/// Gets current size of the stable memory (in WASM pages).
pub fn stable_size() -> u32 {
    CanisterStableMemory::default().stable_size()
}

/// Similar to `stable_size` but with support for 64-bit addressed memory.
pub fn stable64_size() -> u64 {
    CanisterStableMemory::default().stable64_size()
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
    CanisterStableMemory::default().stable_grow(new_pages)
}

/// Similar to `stable_grow` but with support for 64-bit addressed memory.
pub fn stable64_grow(new_pages: u64) -> Result<u64, StableMemoryError> {
    CanisterStableMemory::default().stable64_grow(new_pages)
}

/// Writes data to the stable memory location specified by an offset.
///
/// Warning - this will panic if `offset + buf.len()` exceeds the current size of stable memory.
/// Use `stable_grow` to request more stable memory if needed.
pub fn stable_write(offset: u32, buf: &[u8]) {
    CanisterStableMemory::default().stable_write(offset, buf)
}

/// Similar to `stable_write` but with support for 64-bit addressed memory.
pub fn stable64_write(offset: u64, buf: &[u8]) {
    CanisterStableMemory::default().stable64_write(offset, buf)
}

/// Reads data from the stable memory location specified by an offset.
pub fn stable_read(offset: u32, buf: &mut [u8]) {
    CanisterStableMemory::default().stable_read(offset, buf)
}

/// Similar to `stable_read` but with support for 64-bit addressed memory.
pub fn stable64_read(offset: u64, buf: &mut [u8]) {
    CanisterStableMemory::default().stable64_read(offset, buf)
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

/// A writer to the stable memory which first writes data to a buffer and flushes the buffer to
/// stable memory each time it becomes full. This reduces the number of system calls to
/// `stable64_write` and `stable64_grow` which have relatively large overhead.
struct BufferedStableWriter<M: StableMemory = CanisterStableMemory> {
    /// The offset of the next write.
    offset: u64,

    /// The capacity, in pages.
    capacity: u64,

    /// The buffer to hold data waiting to be written to stable memory
    buffer: Vec<u8>,

    /// The stable memory implementation
    memory: M,
}

impl Default for BufferedStableWriter {
    fn default() -> Self {
        BufferedStableWriter::new(1024 * 1024) // 1MB buffer
    }
}

impl BufferedStableWriter {
    pub fn new(buffer_size: usize) -> BufferedStableWriter {
        let memory = CanisterStableMemory::default();

        Self::with_memory(buffer_size, memory)
    }
}

impl<M: StableMemory> BufferedStableWriter<M> {
    pub fn with_memory(buffer_size: usize, memory: M) -> BufferedStableWriter<M> {
        BufferedStableWriter {
            offset: 0,
            capacity: memory.stable64_size(),
            buffer: Vec::with_capacity(buffer_size),
            memory,
        }
    }

    /// Writes a byte slice to the buffer, flushing the buffer to stable memory if it becomes full.
    ///
    /// The only condition where this will error out is if it cannot grow the memory.
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, StableMemoryError> {
        let buffer_capacity_remaining = self.buffer.capacity() - self.buffer.len();

        match buffer_capacity_remaining.cmp(&buf.len()) {
            // There is enough room in the buffer to store the new bytes.
            Ordering::Greater => {
                self.buffer.extend_from_slice(buf);
            }
            // There is enough room in the buffer to store the new bytes, but now it is full so we
            // need to flush it to stable memory.
            Ordering::Equal => {
                self.buffer.extend_from_slice(buf);
                self.flush()?;
            }
            // The new bytes will not fit in the buffer.
            Ordering::Less => {
                // We can reduce the calls to grow stable memory by growing to the total known
                // length here rather than leaving it up to `flush` which will only grow by up to
                // the length of the buffer.
                let total_capacity_required =
                    self.offset + self.buffer.len() as u64 + buf.len() as u64;
                self.grow_to_capacity_bytes(total_capacity_required)?;

                // If the new bytes exceed the capacity remaining + the starting capacity then we
                // must flush everything now since we will not be able to fit the bytes into the
                // buffer.
                // Otherwise we fill the buffer, flush it, then populate the buffer again with the
                // remaining bytes.
                if buf.len() > self.buffer.capacity() + buffer_capacity_remaining {
                    self.flush()?;
                    self.memory.stable64_write(self.offset, buf);
                    self.offset += buf.len() as u64;
                } else {
                    self.buffer
                        .extend_from_slice(&buf[..buffer_capacity_remaining]);
                    let remaining_to_write = &buf[buffer_capacity_remaining..];
                    self.flush()?;
                    self.buffer.extend_from_slice(remaining_to_write);
                }
            }
        }

        Ok(buf.len())
    }

    /// Attempts to grow the memory by adding new pages.
    pub fn grow(&mut self, added_pages: u64) -> Result<(), StableMemoryError> {
        let old_page_count = self.memory.stable64_grow(added_pages)?;
        self.capacity = old_page_count + added_pages;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), StableMemoryError> {
        if !self.buffer.is_empty() {
            self.grow_to_capacity_bytes(self.offset + self.buffer.len() as u64)?;
            self.memory.stable64_write(self.offset, &self.buffer);
            self.offset += self.buffer.len() as u64;
            self.buffer.clear();
        }

        Ok(())
    }

    fn grow_to_capacity_bytes(
        &mut self,
        required_capacity_bytes: u64,
    ) -> Result<(), StableMemoryError> {
        let required_capacity_pages =
            (required_capacity_bytes + WASM_PAGE_SIZE_IN_BYTES - 1) / WASM_PAGE_SIZE_IN_BYTES;
        let current_pages = self.capacity as u64;
        let additional_pages_required = required_capacity_pages.saturating_sub(current_pages);

        if additional_pages_required > 0 {
            self.grow(additional_pages_required)?;
        }

        Ok(())
    }
}

impl<M: StableMemory> io::Write for BufferedStableWriter<M> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.write(buf)
            .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.flush()
            .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))
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

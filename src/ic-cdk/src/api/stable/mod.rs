//! APIs to manage stable memory.
//!
//! You can check the [Internet Computer Specification](https://smartcontracts.org/docs/interface-spec/index.html#system-api-stable-memory)
//! for a in-depth explanation of stable memory.
mod canister;
mod private;
#[cfg(test)]
mod tests;

pub use canister::CanisterStableMemory;
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
    // SAFETY:
    // `vec`, being mutable and allocated to `size` bytes, is safe to pass to ic0.stable_read with no offset.
    // ic0.stable_read writes to all of `vec[0..size]`, so `set_len` is safe to call with the new size.
    unsafe {
        ic0::stable_read(vec.as_ptr() as i32, 0, size as i32);
        vec.set_len(size);
    }
    vec
}

/// Performs generic IO (read, write, and seek) on stable memory.
///
/// Warning: When using write functionality, this will overwrite any existing
/// data in stable memory as it writes, so ensure you set the `offset` value
/// accordingly if you wish to preserve existing data.
///
/// Will attempt to grow the memory as it writes,
/// and keep offsets and total capacity.

pub struct StableIO<M: StableMemory = CanisterStableMemory, A: private::AddressSize = u32> {
    /// The offset of the next write.
    offset: A,

    /// The capacity, in pages.
    capacity: A,

    /// The stable memory to write data to.
    memory: M,
}

impl Default for StableIO {
    fn default() -> Self {
        Self::with_memory(CanisterStableMemory::default(), 0)
    }
}

// Helper macro to implement StableIO for both 32-bit and 64-bit.
//
// We use a macro here since capturing all the traits required to add and manipulate memory
// addresses with generics becomes cumbersome.
macro_rules! impl_stable_io {
    ($address:ty) => {
        impl<M: private::StableMemory_<$address> + StableMemory> StableIO<M, $address> {
            /// Creates a new `StableIO` which writes to the selected memory
            pub fn with_memory(memory: M, offset: $address) -> Self {
                let capacity = memory.stable_size_();

                Self {
                    offset,
                    capacity,
                    memory,
                }
            }

            /// Returns the offset of the writer
            pub fn offset(&self) -> $address {
                self.offset
            }

            /// Attempts to grow the memory by adding new pages.
            pub fn grow(&mut self, new_pages: $address) -> Result<(), StableMemoryError> {
                let old_page_count = self.memory.stable_grow_(new_pages)?;
                self.capacity = old_page_count + new_pages;
                Ok(())
            }

            /// Writes a byte slice to the buffer.
            ///
            /// The only condition where this will
            /// error out is if it cannot grow the memory.
            pub fn write(&mut self, buf: &[u8]) -> Result<usize, StableMemoryError> {
                let required_capacity_bytes = self.offset + buf.len() as $address;
                let required_capacity_pages =
                    ((required_capacity_bytes + WASM_PAGE_SIZE_IN_BYTES as $address - 1)
                        / WASM_PAGE_SIZE_IN_BYTES as $address);
                let current_pages = self.capacity;
                let additional_pages_required =
                    required_capacity_pages.saturating_sub(current_pages);

                if additional_pages_required > 0 {
                    self.grow(additional_pages_required)?;
                }

                self.memory.stable_write_(self.offset, buf);
                self.offset += buf.len() as $address;
                Ok(buf.len())
            }

            /// Reads data from the stable memory location specified by an offset.
            ///
            /// Note:
            /// The stable memory size is cached on creation of the StableReader.
            /// Therefore, in following scenario, it will get an `OutOfBounds` error:
            /// 1. Create a StableReader
            /// 2. Write some data to the stable memory which causes it grow
            /// 3. call `read()` to read the newly written bytes
            pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, StableMemoryError> {
                let capacity_bytes = self.capacity * WASM_PAGE_SIZE_IN_BYTES as $address;
                let read_buf = if buf.len() as $address + self.offset > capacity_bytes {
                    if self.offset < capacity_bytes {
                        &mut buf[..(capacity_bytes - self.offset) as usize]
                    } else {
                        return Err(StableMemoryError::OutOfBounds);
                    }
                } else {
                    buf
                };
                self.memory.stable_read_(self.offset, read_buf);
                self.offset += read_buf.len() as $address;
                Ok(read_buf.len())
            }

            // Helper used to implement io::Seek
            fn seek(&mut self, offset: io::SeekFrom) -> io::Result<u64> {
                self.offset = match offset {
                    io::SeekFrom::Start(offset) => offset as $address,
                    io::SeekFrom::End(offset) => {
                        ((self.capacity * WASM_PAGE_SIZE_IN_BYTES as $address) as i64 + offset)
                            as $address
                    }
                    io::SeekFrom::Current(offset) => (self.offset as i64 + offset) as $address,
                };

                Ok(self.offset as u64)
            }
        }

        impl<M: private::StableMemory_<$address> + StableMemory> io::Write
            for StableIO<M, $address>
        {
            fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
                self.write(buf)
                    .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))
            }

            fn flush(&mut self) -> Result<(), io::Error> {
                // Noop.
                Ok(())
            }
        }

        impl<M: private::StableMemory_<$address> + StableMemory> io::Read
            for StableIO<M, $address>
        {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
                Self::read(self, buf).or(Ok(0)) // Read defines EOF to be success
            }
        }

        impl<M: private::StableMemory_<$address> + StableMemory> io::Seek
            for StableIO<M, $address>
        {
            fn seek(&mut self, offset: io::SeekFrom) -> io::Result<u64> {
                self.seek(offset)
            }
        }
    };
}

impl_stable_io!(u32);
impl_stable_io!(u64);

/// A writer to the stable memory.
///
/// Warning: This will overwrite any existing data in stable memory as it writes, so ensure you set
/// the `offset` value accordingly if you wish to preserve existing data.
///
/// Will attempt to grow the memory as it writes,
/// and keep offsets and total capacity.
pub struct StableWriter<M: StableMemory = CanisterStableMemory>(StableIO<M, u32>);

#[allow(clippy::derivable_impls)]
impl Default for StableWriter {
    #[inline]
    fn default() -> Self {
        Self(StableIO::default())
    }
}

impl<M: StableMemory> StableWriter<M> {
    /// Creates a new `StableWriter` which writes to the selected memory
    #[inline]
    pub fn with_memory(memory: M, offset: usize) -> Self {
        Self(StableIO::<M, u32>::with_memory(memory, offset as u32))
    }

    /// Returns the offset of the writer
    #[inline]
    pub fn offset(&self) -> usize {
        self.0.offset() as usize
    }

    /// Attempts to grow the memory by adding new pages.
    #[inline]
    pub fn grow(&mut self, new_pages: u32) -> Result<(), StableMemoryError> {
        self.0.grow(new_pages)
    }

    /// Writes a byte slice to the buffer.
    ///
    /// The only condition where this will
    /// error out is if it cannot grow the memory.
    #[inline]
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, StableMemoryError> {
        self.0.write(buf)
    }
}

impl<M: StableMemory> io::Write for StableWriter<M> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        io::Write::write(&mut self.0, buf)
    }

    #[inline]
    fn flush(&mut self) -> Result<(), io::Error> {
        io::Write::flush(&mut self.0)
    }
}

impl<M: StableMemory> io::Seek for StableWriter<M> {
    #[inline]
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        io::Seek::seek(&mut self.0, pos)
    }
}

impl<M: StableMemory> From<StableIO<M>> for StableWriter<M> {
    fn from(io: StableIO<M>) -> Self {
        Self(io)
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

    /// Returns the offset of the writer
    pub fn offset(&self) -> usize {
        self.inner.get_ref().offset()
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

impl<M: StableMemory> io::Seek for BufferedStableWriter<M> {
    #[inline]
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        io::Seek::seek(&mut self.inner, pos)
    }
}

// A reader to the stable memory.
///
/// Keeps an offset and reads off stable memory consecutively.
pub struct StableReader<M: StableMemory = CanisterStableMemory>(StableIO<M, u32>);

#[allow(clippy::derivable_impls)]
impl Default for StableReader {
    fn default() -> Self {
        Self(StableIO::default())
    }
}

impl<M: StableMemory> StableReader<M> {
    /// Creates a new `StableReader` which reads from the selected memory
    #[inline]
    pub fn with_memory(memory: M, offset: usize) -> Self {
        Self(StableIO::<M, u32>::with_memory(memory, offset as u32))
    }

    /// Returns the offset of the reader
    #[inline]
    pub fn offset(&self) -> usize {
        self.0.offset() as usize
    }

    /// Reads data from the stable memory location specified by an offset.
    ///
    /// Note:
    /// The stable memory size is cached on creation of the StableReader.
    /// Therefore, in following scenario, it will get an `OutOfBounds` error:
    /// 1. Create a StableReader
    /// 2. Write some data to the stable memory which causes it grow
    /// 3. call `read()` to read the newly written bytes
    #[inline]
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, StableMemoryError> {
        self.0.read(buf)
    }
}

impl<M: StableMemory> io::Read for StableReader<M> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        io::Read::read(&mut self.0, buf)
    }
}

impl<M: StableMemory> io::Seek for StableReader<M> {
    #[inline]
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        io::Seek::seek(&mut self.0, pos)
    }
}

impl<M: StableMemory> From<StableIO<M>> for StableReader<M> {
    fn from(io: StableIO<M>) -> Self {
        Self(io)
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

    /// Returns the offset of the reader
    pub fn offset(&self) -> usize {
        self.inner.get_ref().offset()
    }
}

impl<M: StableMemory> io::Read for BufferedStableReader<M> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<M: StableMemory> io::Seek for BufferedStableReader<M> {
    #[inline]
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        io::Seek::seek(&mut self.inner, pos)
    }
}

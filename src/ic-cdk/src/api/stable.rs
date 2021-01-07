use std::io;

pub fn stable_size() -> u32 {
    unsafe { super::ic0::stable_size() as u32 }
}

pub struct StableMemoryError();

/// Attempt to grow the stable memory by `new_pages` (added pages).
/// Returns an error if it wasn't possible. Otherwise, returns the previous
/// size that was reserved.
///
/// ## Notes
/// Pages are 64KiB in WASM.
pub fn stable_grow(new_pages: u32) -> Result<u32, StableMemoryError> {
    unsafe {
        match super::ic0::stable_grow(new_pages as i32) {
            -1 => Err(StableMemoryError()),
            x => Ok(x as u32),
        }
    }
}

pub fn stable_write(offset: u32, buf: &[u8]) {
    unsafe {
        super::ic0::stable_write(offset as i32, buf.as_ptr() as i32, buf.len() as i32);
    }
}

pub fn stable_read(offset: u32, buf: &mut [u8]) {
    unsafe {
        super::ic0::stable_read(buf.as_ptr() as i32, offset as i32, buf.len() as i32);
    }
}

/// Returns a copy of the stable memory. This will map the whole memory (even if not all of it
/// has been written to).
pub fn stable_bytes() -> Vec<u8> {
    let size = (stable_size() as usize) << 16;
    let mut vec = Vec::with_capacity(size);
    unsafe {
        vec.set_len(size);
    }

    stable_read(0, vec.as_mut_slice());

    vec
}

/// A writer to the stable memory. Will attempt to grow the memory as it
/// writes, and keep offsets and total capacity.
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
    /// Attempt to grow the memory by adding new pages.
    pub fn grow(&mut self, added_pages: u32) -> Result<(), StableMemoryError> {
        let old_page_count = stable_grow(added_pages)?;
        self.capacity = old_page_count + added_pages;
        Ok(())
    }

    /// Write a byte slice to the buffer. The only condition where this will
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
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Out Of Memory"))
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        // Noop.
        Ok(())
    }
}

/// A reader to the stable memory. Keeps an offset and reads off stable memory
/// consecutively.
pub struct StableReader {
    /// The offset of the next write.
    offset: usize,
}

impl Default for StableReader {
    fn default() -> Self {
        Self { offset: 0 }
    }
}

impl StableReader {
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, StableMemoryError> {
        stable_read(self.offset as u32, buf);
        self.offset += buf.len();
        Ok(buf.len())
    }
}

impl io::Read for StableReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.read(buf)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Unexpected error."))
    }
}

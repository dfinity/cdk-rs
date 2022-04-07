use std::cell::RefCell;
use super::*;

thread_local! {
    static MEMORY: RefCell<Vec<u8>> = RefCell::default();
}

pub struct TestStableMemory {}

impl StableMemory for TestStableMemory {
    fn stable_size() -> u32 {
        let bytes_len = MEMORY.with(|m| m.borrow().len()) as u32;
        (bytes_len + WASM_PAGE_SIZE_IN_BYTES - 1) / WASM_PAGE_SIZE_IN_BYTES
    }

    fn stable64_size() -> u64 {
        TestStableMemory::stable_size() as u64
    }

    fn stable_grow(new_pages: u32) -> Result<u32, StableMemoryError> {
        let new_bytes = (new_pages as usize * WASM_PAGE_SIZE_IN_BYTES as usize);

        MEMORY.with(|m| {
            let mut vec = m.borrow_mut();
            let previous_len = vec.len();
            let new_len = vec.len() + new_bytes;
            vec.resize(new_len, 0);
            Ok(previous_len as u32)
        })
    }

    fn stable64_grow(new_pages: u64) -> Result<u64, StableMemoryError> {
        TestStableMemory::stable_grow(new_pages as u32).map(|len| len as u64)
    }

    fn stable_write(offset: u32, buf: &[u8]) {
        let offset = offset as usize;

        MEMORY.with(|m| {
            let mut vec = m.borrow_mut();
            if offset + buf.len() > vec.len() {
                panic!("stable memory out of bounds".to_string());
            }
            vec[offset..(offset + buf.len())] = *buf;
        })
    }

    fn stable64_write(offset: u64, buf: &[u8]) {
        TestStableMemory::stable_write(offset as u32, buf)
    }

    fn stable_read(offset: u32, buf: &mut [u8]) {
        let offset = offset as usize;

        MEMORY.with(|m| {
            let vec = m.borrow();
            if offset + buf.len() < vec.len() {
                panic!("stable memory out of bounds".to_string());
            }
            buf[..vec.len()].copy_from_slice(&vec[offset..]);
        })
    }

    fn stable64_read(offset: u64, buf: &mut [u8]) {
        TestStableMemory::stable_read(offset as u32, buf)
    }
}

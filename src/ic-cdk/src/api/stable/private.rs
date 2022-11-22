//! Private module for traits that provide implementations for both u32 and u64 address space.

use super::*;
pub trait AddressSize {}
impl AddressSize for u64 {}
impl AddressSize for u32 {}

// An internal helper trait to have stable memory API specific to an address size
pub trait StableMemory_<A: AddressSize> {
    fn stable_size_(&self) -> A;
    fn stable_grow_(&self, new_pages: A) -> Result<A, StableMemoryError>;
    fn stable_write_(&self, offset: A, buf: &[u8]);
    fn stable_read_(&self, offset: A, buf: &mut [u8]);
}

// Blanket implementation for 32-bit addresses for any stable memory implementation
impl<M: StableMemory> StableMemory_<u32> for M {
    fn stable_read_(&self, offset: u32, buf: &mut [u8]) {
        StableMemory::stable_read(self, offset, buf)
    }

    fn stable_grow_(&self, new_pages: u32) -> Result<u32, StableMemoryError> {
        StableMemory::stable_grow(self, new_pages)
    }

    fn stable_size_(&self) -> u32 {
        StableMemory::stable_size(self)
    }

    fn stable_write_(&self, offset: u32, buf: &[u8]) {
        StableMemory::stable_write(self, offset, buf)
    }
}

// Blanket implementation for 64-bit addresses for any stable memory implementation
impl<M: super::StableMemory> StableMemory_<u64> for M {
    fn stable_read_(&self, offset: u64, buf: &mut [u8]) {
        StableMemory::stable64_read(self, offset, buf)
    }

    fn stable_grow_(&self, new_pages: u64) -> Result<u64, StableMemoryError> {
        StableMemory::stable64_grow(self, new_pages)
    }

    fn stable_size_(&self) -> u64 {
        StableMemory::stable64_size(self)
    }

    fn stable_write_(&self, offset: u64, buf: &[u8]) {
        StableMemory::stable64_write(self, offset, buf)
    }
}

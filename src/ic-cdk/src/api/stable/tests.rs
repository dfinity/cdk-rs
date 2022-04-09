use super::*;
use std::rc::Rc;
use std::sync::Mutex;

#[derive(Default)]
pub struct TestStableMemory {
    memory: Rc<Mutex<Vec<u8>>>,
}

impl TestStableMemory {
    pub fn new(memory: Rc<Mutex<Vec<u8>>>) -> TestStableMemory {
        TestStableMemory { memory }
    }
}

impl StableMemory for TestStableMemory {
    fn stable_size(&self) -> u32 {
        let bytes_len = self.memory.lock().unwrap().len();
        let page_size = WASM_PAGE_SIZE_IN_BYTES as usize;
        ((bytes_len + page_size - 1) / page_size) as u32
    }

    fn stable64_size(&self) -> u64 {
        self.stable_size() as u64
    }

    fn stable_grow(&self, new_pages: u32) -> Result<u32, StableMemoryError> {
        let new_bytes = new_pages as usize * WASM_PAGE_SIZE_IN_BYTES as usize;

        let mut vec = self.memory.lock().unwrap();
        let previous_len = vec.len();
        let new_len = vec.len() + new_bytes;
        vec.resize(new_len, 0);
        Ok(previous_len as u32)
    }

    fn stable64_grow(&self, new_pages: u64) -> Result<u64, StableMemoryError> {
        self.stable_grow(new_pages as u32).map(|len| len as u64)
    }

    fn stable_write(&self, offset: u32, buf: &[u8]) {
        let offset = offset as usize;

        let mut vec = self.memory.lock().unwrap();
        if offset + buf.len() > vec.len() {
            panic!("stable memory out of bounds");
        }
        vec[offset..(offset + buf.len())].clone_from_slice(buf);
    }

    fn stable64_write(&self, offset: u64, buf: &[u8]) {
        self.stable_write(offset as u32, buf)
    }

    fn stable_read(&self, offset: u32, buf: &mut [u8]) {
        let offset = offset as usize;

        let vec = self.memory.lock().unwrap();
        if offset + buf.len() < vec.len() {
            panic!("stable memory out of bounds");
        }
        buf[..vec.len()].copy_from_slice(&vec[offset..]);
    }

    fn stable64_read(&self, offset: u64, buf: &mut [u8]) {
        self.stable_read(offset as u32, buf)
    }
}

mod stable_writer_tests {
    use super::*;
    use rstest::rstest;
    use std::io::Write;

    #[rstest]
    #[case(None)]
    #[case(Some(1))]
    #[case(Some(10))]
    #[case(Some(100))]
    #[case(Some(1000))]
    fn write_single_slice(#[case] buffer_size: Option<usize>) {
        let memory = Rc::new(Mutex::new(Vec::new()));
        let mut writer = build_writer(TestStableMemory::new(memory.clone()), buffer_size);

        let bytes = vec![1; 100];

        writer.write_all(&bytes).unwrap();
        writer.flush().unwrap();

        let result = &*memory.lock().unwrap();

        assert_eq!(bytes, result[..bytes.len()]);
    }

    #[rstest]
    #[case(None)]
    #[case(Some(1))]
    #[case(Some(10))]
    #[case(Some(100))]
    #[case(Some(1000))]
    fn write_many_slices(#[case] buffer_size: Option<usize>) {
        let memory = Rc::new(Mutex::new(Vec::new()));
        let mut writer = build_writer(TestStableMemory::new(memory.clone()), buffer_size);

        for i in 1..100 {
            let bytes = vec![i as u8; i];
            writer.write_all(&bytes).unwrap();
        }
        writer.flush().unwrap();

        let result = &*memory.lock().unwrap();

        let mut offset = 0;
        for i in 1..100 {
            let bytes = &result[offset..offset + i];
            assert_eq!(bytes, vec![i as u8; i]);
            offset += i;
        }
    }

    fn build_writer(memory: TestStableMemory, buffer_size: Option<usize>) -> Box<dyn Write> {
        let writer = StableWriter::with_memory(memory, 0);
        if let Some(buffer_size) = buffer_size {
            Box::new(BufferedStableWriter::with_writer(buffer_size, writer))
        } else {
            Box::new(writer)
        }
    }
}

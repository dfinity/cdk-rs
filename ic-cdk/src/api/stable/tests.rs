use super::*;
use std::rc::Rc;
use std::sync::Mutex;

#[derive(Default)]
pub struct TestStableMemory {
    memory: Rc<Mutex<Vec<u8>>>,
}

impl TestStableMemory {
    pub fn new(memory: Rc<Mutex<Vec<u8>>>) -> TestStableMemory {
        let bytes_len = memory.lock().unwrap().len();
        if bytes_len > 0 {
            let pages_required = pages_required(bytes_len);
            let bytes_required = pages_required * WASM_PAGE_SIZE_IN_BYTES;
            memory
                .lock()
                .unwrap()
                .resize(bytes_required.try_into().unwrap(), 0);
        }

        TestStableMemory { memory }
    }
}

impl StableMemory for TestStableMemory {
    fn stable_size(&self) -> u64 {
        let bytes_len = self.memory.lock().unwrap().len();
        pages_required(bytes_len)
    }

    fn stable_grow(&self, new_pages: u64) -> Result<u64, StableMemoryError> {
        let new_bytes = new_pages * WASM_PAGE_SIZE_IN_BYTES;

        let mut vec = self.memory.lock().unwrap();
        let previous_len = vec.len() as u64;
        let new_len = vec.len() as u64 + new_bytes;
        vec.resize(new_len.try_into().unwrap(), 0);
        Ok(previous_len / WASM_PAGE_SIZE_IN_BYTES)
    }

    fn stable_write(&self, offset: u64, buf: &[u8]) {
        let offset = offset as usize;

        let mut vec = self.memory.lock().unwrap();
        if offset + buf.len() > vec.len() {
            panic!("stable memory out of bounds");
        }
        vec[offset..(offset + buf.len())].clone_from_slice(buf);
    }

    fn stable_read(&self, offset: u64, buf: &mut [u8]) {
        let offset = offset as usize;

        let vec = self.memory.lock().unwrap();
        let count_to_copy = buf.len();

        buf[..count_to_copy].copy_from_slice(&vec[offset..offset + count_to_copy]);
    }
}

fn pages_required(bytes_len: usize) -> u64 {
    let page_size = WASM_PAGE_SIZE_IN_BYTES;
    (bytes_len as u64 + page_size - 1) / page_size
}

mod stable_writer_tests {
    use super::*;
    use rstest::rstest;
    use std::io::{Seek, Write};

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

    #[rstest]
    #[case(None)]
    #[case(Some(1))]
    #[case(Some(10))]
    #[case(Some(100))]
    #[case(Some(1000))]
    fn ensure_only_requests_min_number_of_pages_required(#[case] buffer_size: Option<usize>) {
        let memory = Rc::new(Mutex::new(Vec::new()));
        let mut writer = build_writer(TestStableMemory::new(memory.clone()), buffer_size);

        let mut total_bytes = 0;
        for i in 1..10000 {
            let bytes = vec![i as u8; i];
            writer.write_all(&bytes).unwrap();
            total_bytes += i;
        }
        writer.flush().unwrap();

        let capacity_pages = TestStableMemory::new(memory).stable_size();
        let min_pages_required =
            (total_bytes as u64 + WASM_PAGE_SIZE_IN_BYTES - 1) / WASM_PAGE_SIZE_IN_BYTES;

        assert_eq!(capacity_pages, min_pages_required);
    }

    #[test]
    fn check_offset() {
        const WRITE_SIZE: usize = 1025;

        let memory = Rc::new(Mutex::new(Vec::new()));
        let mut writer = StableWriter::with_memory(TestStableMemory::new(memory.clone()), 0);
        assert_eq!(writer.offset(), 0);
        assert_eq!(writer.write(&vec![0; WRITE_SIZE]).unwrap(), WRITE_SIZE);
        assert_eq!(writer.offset(), WRITE_SIZE as u64);

        let mut writer = BufferedStableWriter::with_writer(
            WRITE_SIZE - 1,
            StableWriter::with_memory(TestStableMemory::new(memory), 0),
        );
        assert_eq!(writer.offset(), 0);
        assert_eq!(writer.write(&vec![0; WRITE_SIZE]).unwrap(), WRITE_SIZE);
        assert_eq!(writer.offset(), WRITE_SIZE as u64);
    }

    #[test]
    fn test_seek() {
        let memory = Rc::new(Mutex::new(Vec::new()));
        let mut writer = StableWriter::with_memory(TestStableMemory::new(memory.clone()), 0);
        writer
            .seek(std::io::SeekFrom::Start(WASM_PAGE_SIZE_IN_BYTES))
            .unwrap();
        assert_eq!(writer.stream_position().unwrap(), WASM_PAGE_SIZE_IN_BYTES);
        assert_eq!(writer.write(&[1_u8]).unwrap(), 1);
        assert_eq!(
            writer.seek(std::io::SeekFrom::End(0)).unwrap(),
            WASM_PAGE_SIZE_IN_BYTES * 2
        );
        let capacity_pages = TestStableMemory::new(memory).stable_size();
        assert_eq!(capacity_pages, 2);
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

mod stable_reader_tests {
    use super::*;
    use rstest::rstest;
    use std::io::{Read, Seek};

    #[rstest]
    #[case(None)]
    #[case(Some(1))]
    #[case(Some(10))]
    #[case(Some(100))]
    #[case(Some(1000))]
    fn reads_all_bytes(#[case] buffer_size: Option<usize>) {
        let input = vec![1; 10_000];
        let memory = Rc::new(Mutex::new(input.clone()));
        let mut reader = build_reader(TestStableMemory::new(memory), buffer_size);

        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();

        assert_eq!(input, output[..input.len()]);
    }

    #[test]
    fn check_offset() {
        const READ_SIZE: usize = 1025;

        let memory = Rc::new(Mutex::new(vec![1; READ_SIZE]));
        let mut reader = StableReader::with_memory(TestStableMemory::new(memory.clone()), 0);
        assert_eq!(reader.offset(), 0);
        let mut bytes = vec![0; READ_SIZE];
        assert_eq!(reader.read(&mut bytes).unwrap(), READ_SIZE);
        assert_eq!(reader.offset(), READ_SIZE as u64);

        let mut reader = BufferedStableReader::with_reader(
            READ_SIZE - 1,
            StableReader::with_memory(TestStableMemory::new(memory), 0),
        );
        assert_eq!(reader.offset(), 0);
        let mut bytes = vec![0; READ_SIZE];
        assert_eq!(reader.read(&mut bytes).unwrap(), READ_SIZE);
        assert_eq!(reader.offset(), READ_SIZE as u64);
    }

    #[test]
    fn test_seek() {
        const SIZE: usize = 1025;
        let memory = Rc::new(Mutex::new((0..SIZE).map(|v| v as u8).collect::<Vec<u8>>()));
        let mut reader = StableReader::with_memory(TestStableMemory::new(memory), 0);
        let mut bytes = vec![0_u8; 1];

        const OFFSET: usize = 200;
        reader
            .seek(std::io::SeekFrom::Start(OFFSET as u64))
            .unwrap();
        assert_eq!(reader.stream_position().unwrap() as usize, OFFSET);
        assert_eq!(reader.read(&mut bytes).unwrap(), 1);
        assert_eq!(&bytes, &[OFFSET as u8]);
        assert_eq!(
            reader.seek(std::io::SeekFrom::End(0)).unwrap(),
            WASM_PAGE_SIZE_IN_BYTES
        );
        reader
            .seek(std::io::SeekFrom::Start(WASM_PAGE_SIZE_IN_BYTES * 2))
            .unwrap();
        // out of bounds so should fail
        assert!(reader.read(&mut bytes).is_err());
    }

    fn build_reader(memory: TestStableMemory, buffer_size: Option<usize>) -> Box<dyn Read> {
        let reader = StableReader::with_memory(memory, 0);
        if let Some(buffer_size) = buffer_size {
            Box::new(BufferedStableReader::with_reader(buffer_size, reader))
        } else {
            Box::new(reader)
        }
    }
}

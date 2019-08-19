use crate::{errors::*, os, strategy};

use failure::Fail;
use memmap::Mmap;
use std::{fs::File, io, path::Path};

pub const MIN_READ_BUF_SIZE: usize = os::PAGE_SIZE;
pub const MAX_READ_BUF_SIZE: usize = 4 * 1024 * 1024;

/// Allocate a memory buffer on the stack and initialize it for the reader
///
/// This macro takes a `Read` as first parameter and optionally a buffer size as second parameter.
/// If the second parameter is ommited, the buffer is allocated with size of `MAX_READ_BUF_SIZE`.
#[macro_export]
macro_rules! prepare_buf {
    ($reader:ident, $size:tt) => {
        unsafe {
            let buf = std::mem::MaybeUninit::<[u8; $size]>::uninit();
            buf.assume_init()
        };
    };
    ($reader:ident) => {
        prepare_buf!($reader, MAX_READ_BUF_SIZE)
    };
}

/// `FastFile` is the main API for fast reading and writing files
pub struct FastFile {}

impl FastFile {
    /// Open a new `FastFile` for reading similar to `std::io::File::open()`
    pub fn read<P: AsRef<Path>>(path: P) -> Result<FastFileReaderBuilder> {
        let file = File::open(path).map_err(|e| e.context(ErrorKind::FileOpFailed))?;
        let ff = FastFileReaderBuilder {
            file,
            size: None,
            size_hint: None,
        };

        Ok(ff)
    }
}

/// `FastFileReaderBuilder` is a builder for a FastFileReader
pub struct FastFileReaderBuilder {
    pub file:      File,
    pub size:      Option<u64>,
    pub size_hint: Option<u64>,
}

impl FastFileReaderBuilder {
    pub fn with_size(self, size: u64) -> Self {
        FastFileReaderBuilder {
            size: Some(size),
            ..self
        }
    }

    pub fn with_size_hint(self, size_hint: u64) -> Self {
        FastFileReaderBuilder {
            size_hint: Some(size_hint),
            ..self
        }
    }

    pub fn open_with_strategy<T: strategy::ReaderStrategy>(self, reader_strategy: &T) -> Result<FastFileReader> {
        reader_strategy.get_reader(self)
    }

    pub fn open(self) -> Result<FastFileReader> {
        let reader_strategy = strategy::DefaultReaderStrategy {};
        self.open_with_strategy(&reader_strategy)
    }
}

/// Backing Reader for FastFileReader
pub enum BackingReader {
    File(File),
    Mmap(File, std::io::Cursor<Mmap>),
}

impl BackingReader {
    pub fn file(file: File) -> Result<BackingReader> { Ok(BackingReader::File(file)) }

    pub fn mmap(file: File) -> Result<BackingReader> {
        let mmap = unsafe { Mmap::map(&file).map_err(|e| e.context(ErrorKind::FileOpFailed))? };
        Ok(BackingReader::Mmap(file, std::io::Cursor::new(mmap)))
    }
}

impl io::Read for BackingReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            BackingReader::File(file) => file.read(buf),
            BackingReader::Mmap(_, mmap) => mmap.read(buf),
        }
    }
}

/// `FastFileReader` is a readable (`std::io::Read`) FastFile
pub struct FastFileReader {
    inner:  BackingReader,
    size:   u64,
    buffer: Option<Vec<u8>>,
}

impl FastFileReader {
    pub fn new(inner: BackingReader, size: u64) -> FastFileReader {
        FastFileReader {
            inner,
            size,
            buffer: None,
        }
    }

    pub fn size(&self) -> u64 { self.size }

    fn init_buffer(&mut self) {
        let buf_size = optimal_buffer_size(self.size);
        let mut vec: Vec<u8> = Vec::with_capacity(buf_size);
        unsafe {
            vec.set_len(buf_size);
        }
        // let buf = vec.as_mut_slice();
        // unsafe {
        // self.inner.initializer().initialize(&mut *buf);
        // }
        self.buffer = Some(vec);
    }
}

// Computes the optimal buffer size for a specified file size aligned to the system's page size.
pub fn optimal_buffer_size(file_size: u64) -> usize {
    let size = file_size as usize;
    let suggestion = ((size + os::PAGE_SIZE - 1) / os::PAGE_SIZE) * os::PAGE_SIZE;
    let suggestion = MAX_READ_BUF_SIZE.min(suggestion);
    MIN_READ_BUF_SIZE.max(suggestion)
}

pub trait FastFileRead {
    fn read(&mut self) -> io::Result<&[u8]>;

    fn read_to_end(&mut self) -> io::Result<&[u8]>;
}

impl FastFileRead for FastFileReader {
    fn read(&mut self) -> io::Result<&[u8]> {
        use std::io::Read;

        if self.buffer.is_none() {
            self.init_buffer();
        }
        let vec = self.buffer.as_mut().unwrap(); // Safe, bc we checked above
        let buf = vec.as_mut_slice();

        let n = self.inner.read(&mut buf[..])?;

        Ok(&buf[0..n])
    }

    fn read_to_end(&mut self) -> io::Result<&[u8]> {
        use std::io::Read;

        if self.buffer.is_none() {
            self.init_buffer();
        }
        let mut vec = self.buffer.as_mut().unwrap(); // Safe, bc we checked above

        // `Read::read_to_end` _appends_ to the specified buffer; so we need to set the len to 0
        // first
        unsafe {
            vec.set_len(0);
        }

        let n = self.inner.read_to_end(&mut vec)?;
        let buf = vec.as_mut_slice();

        Ok(&buf[0..n])
    }
}

impl io::Read for FastFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> { self.inner.read(buf) }
}

#[cfg(test)]
mod tests {
    use super::{
        optimal_buffer_size,
        os::PAGE_SIZE,
        strategy,
        BackingReader,
        FastFile,
        FastFileReader,
        FastFileReaderBuilder,
        Result,
        MAX_READ_BUF_SIZE,
        MIN_READ_BUF_SIZE,
    };

    use rand::{rngs::SmallRng, Rng, SeedableRng};
    use ring::digest::{Context, Digest, SHA256};

    use spectral::prelude::*;

    #[test]
    fn test_optimal_buffer_size_1024() {
        asserting("size = 0")
            .that(&optimal_buffer_size(0))
            .is_equal_to(&MIN_READ_BUF_SIZE);
        asserting("size = 1")
            .that(&optimal_buffer_size(1))
            .is_equal_to(&MIN_READ_BUF_SIZE);
        asserting("size = 1023")
            .that(&optimal_buffer_size(1023))
            .is_equal_to(&MIN_READ_BUF_SIZE);
        asserting("size = 1024")
            .that(&optimal_buffer_size(1024))
            .is_equal_to(&MIN_READ_BUF_SIZE);
    }

    #[test]
    fn test_optimal_buffer_size_page_size() {
        asserting("size = PAGE_SIZE - 1")
            .that(&optimal_buffer_size(PAGE_SIZE as u64 - 1))
            .is_equal_to(&PAGE_SIZE);
        asserting("size = PAGE_SIZE")
            .that(&optimal_buffer_size(PAGE_SIZE as u64))
            .is_equal_to(&PAGE_SIZE);
        asserting("size = PAGE_SIZE + 1")
            .that(&optimal_buffer_size(PAGE_SIZE as u64 + 1))
            .is_equal_to(2 * PAGE_SIZE);
        asserting("size = 2*PAGE_SIZE + 1")
            .that(&optimal_buffer_size(2 * PAGE_SIZE as u64 + 1))
            .is_equal_to(3 * PAGE_SIZE);
    }

    #[test]
    fn test_optimal_buffer_size_min_read_buf_size() {
        asserting("size = MIN_READ_BUF_SIZE - 1")
            .that(&optimal_buffer_size(MIN_READ_BUF_SIZE as u64 - 1))
            .is_equal_to(&MIN_READ_BUF_SIZE);
        asserting("size = MIN_READ_BUF_SIZE")
            .that(&optimal_buffer_size(MIN_READ_BUF_SIZE as u64))
            .is_equal_to(&MIN_READ_BUF_SIZE);
        asserting("size = MIN_READ_BUF_SIZE + 1")
            .that(&optimal_buffer_size(MIN_READ_BUF_SIZE as u64 + 1))
            .is_equal_to(&(2 * MIN_READ_BUF_SIZE));
    }

    #[test]
    fn test_optimal_buffer_size_max_read_buf_size() {
        asserting("size = MAX_READ_BUF_SIZE - 1")
            .that(&optimal_buffer_size(MAX_READ_BUF_SIZE as u64 - 1))
            .is_equal_to(&MAX_READ_BUF_SIZE);
        asserting("size = MAX_READ_BUF_SIZE")
            .that(&optimal_buffer_size(MAX_READ_BUF_SIZE as u64))
            .is_equal_to(&MAX_READ_BUF_SIZE);
        asserting("size = MAX_READ_BUF_SIZE + 1")
            .that(&optimal_buffer_size(MAX_READ_BUF_SIZE as u64 + 1))
            .is_equal_to(&(MAX_READ_BUF_SIZE));
    }

    mod read {
        use super::*;

        use std::io::Read;

        #[test]
        fn fastfilereader_read_correctly_with_file_backend() {
            let reader_strategy = TestFileReaderStragegy {};
            fastfilereader_reads_correctly_tester(&reader_strategy);
        }

        #[test]
        fn fastfilereader_read_correctly_with_mmap_backend() {
            let reader_strategy = TestMmapReaderStragegy {};
            fastfilereader_reads_correctly_tester(&reader_strategy);
        }

        fn fastfilereader_reads_correctly_tester<T: strategy::ReaderStrategy>(reader_strategy: &T) {
            verify_reader(reader_strategy, |ffr: &mut FastFileReader| {
                let mut len = 0u64;
                let mut digest = Context::new(&SHA256);
                let mut buf = prepare_buf!(ffr, 4096);
                loop {
                    let n = ffr.read(&mut buf).expect("Failed to fastread file");
                    if n == 0 {
                        break;
                    };
                    len += n as u64;
                    digest.update(&buf[0..n]);
                }
                let digest = digest.finish();
                (len, digest)
            });
        }
    }

    mod fast_read {
        use super::*;

        use crate::fastfile::FastFileRead;

        #[test]
        fn fastfilereader_reads_correctly_with_file_backend() {
            let reader_strategy = TestFileReaderStragegy {};
            fastfilereader_reads_correctly_tester(&reader_strategy);
        }

        #[test]
        fn fastfilereader_reads_correctly_with_mmap_backend() {
            let reader_strategy = TestMmapReaderStragegy {};
            fastfilereader_reads_correctly_tester(&reader_strategy);
        }

        fn fastfilereader_reads_correctly_tester<T: strategy::ReaderStrategy>(reader_strategy: &T) {
            verify_reader(reader_strategy, |ffr: &mut FastFileReader| {
                let mut len = 0u64;
                let mut digest = Context::new(&SHA256);
                loop {
                    let buf = ffr.read().expect("Failed to fastread file");
                    if buf.is_empty() {
                        break;
                    };
                    len += buf.len() as u64;
                    digest.update(buf);
                }
                let digest = digest.finish();
                (len, digest)
            });
        }
    }

    fn verify_reader<T: strategy::ReaderStrategy, F: Fn(&mut FastFileReader) -> (u64, Digest)>(
        reader_strategy: &T,
        reader: F,
    ) {
        let mut rng = SmallRng::from_entropy();
        let size = rng.gen_range(1024 * 1024 + 1, 2 * 1024 * 1024);

        let path = fastfile_benches::utils::create_random_test_file(size).expect("Failed to create test file");
        let mut ffr = FastFile::read(&path)
            .expect("Failed to create FastFileReaderBuilder")
            .open_with_strategy(reader_strategy)
            .expect("Failed to open path as FastFile");

        let (len, digest) = reader(&mut ffr);

        assert_eq!(len, ffr.size(), "Read bytes differ from file size");

        let expected_digest =
            fastfile_benches::utils::get_digest_for_path(&path).expect("Failed to compute expected digest");
        assert_eq!(
            digest.as_ref(),
            expected_digest.as_ref(),
            "Computed digest differes from expected digest"
        );
    }

    struct TestFileReaderStragegy {}
    impl strategy::ReaderStrategy for TestFileReaderStragegy {
        fn get_reader(&self, ffrb: FastFileReaderBuilder) -> Result<FastFileReader> {
            let FastFileReaderBuilder { file, .. } = ffrb;
            let size = file.metadata().unwrap().len();
            let inner = BackingReader::file(file)?;

            Ok(FastFileReader::new(inner, size))
        }
    }

    struct TestMmapReaderStragegy {}
    impl strategy::ReaderStrategy for TestMmapReaderStragegy {
        fn get_reader(&self, ffrb: FastFileReaderBuilder) -> Result<FastFileReader> {
            let FastFileReaderBuilder { file, .. } = ffrb;
            let size = file.metadata().unwrap().len();
            let inner = BackingReader::mmap(file)?;

            Ok(FastFileReader::new(inner, size))
        }
    }
}

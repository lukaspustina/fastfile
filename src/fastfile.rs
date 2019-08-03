use crate::{errors::*, strategy};

use failure::Fail;
use memmap::Mmap;
use std::{fs::File, io, path::Path};

pub const MAX_READ_BUF_SIZE: usize = 64 * 1024;

/// Allocate a memory buffer on the stack and initialize it for the reader
///
/// This macro takes a `Read` as first parameter and optionally a buffer size as second parameter.
/// If the second parameter is ommited, the buffer is allocated with size of `MAX_READ_BUF_SIZE`.
#[macro_export]
macro_rules! prepare_buf {
    ($reader:ident, $size:tt) => {
        unsafe {
            let mut buf = std::mem::MaybeUninit::<[u8; $size]>::uninit();
            $reader.initializer().initialize(&mut *buf.as_mut_ptr());
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
            file: Some(file),
            ..Default::default()
        };

        Ok(ff)
    }
}

/// `FastFileReaderBuilder` is a builder for a FastFileReader
pub struct FastFileReaderBuilder {
    pub file: Option<File>,
    pub size: Option<u64>,
}

impl Default for FastFileReaderBuilder {
    fn default() -> Self {
        FastFileReaderBuilder {
            file: None,
            size: None,
        }
    }
}

impl FastFileReaderBuilder {
    pub fn set_size(self, size: u64) -> Self {
        FastFileReaderBuilder {
            size: Some(size),
            ..self
        }
    }

    pub fn open_with_strategy<T: strategy::ReaderStrategy>(
        self,
        reader_strategy: &T,
    ) -> Result<FastFileReader> {
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
    pub fn file(file: File) -> Result<BackingReader> {
        Ok(BackingReader::File(file))
    }

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
    inner: BackingReader,
    size: u64,
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

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn optimal_buffer_size(&self) -> usize {
        MAX_READ_BUF_SIZE.min(self.size.next_power_of_two() as usize)
    }

    fn init_buffer(&mut self) {
        use std::io::Read;

        let mut vec: Vec<u8> = Vec::with_capacity(MAX_READ_BUF_SIZE);
        unsafe {
            vec.set_len(MAX_READ_BUF_SIZE);
        }
        let buf = vec.as_mut_slice();
        unsafe {
            self.inner.initializer().initialize(&mut *buf);
        }
        self.buffer = Some(vec);
    }
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

        let n = self.inner.read(&mut buf[0..MAX_READ_BUF_SIZE])?;

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
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::{strategy, BackingReader, FastFile, FastFileReader, FastFileReaderBuilder, Result};

    use spectral::prelude::*;

    mod read {
        use super::*;

        use std::io::Read;

        #[test]
        fn fastfilereader_read_correctly_with_file_backend() {
            let reader_strategy = TestFileReaderStragegy {};

            fastfilereader_read_correctly_tester(&reader_strategy);
        }

        #[test]
        fn fastfilereader_read_correctly_with_mmap_backend() {
            let reader_strategy = TestMmapReaderStragegy {};

            fastfilereader_read_correctly_tester(&reader_strategy);
        }

        fn fastfilereader_read_correctly_tester<T: strategy::ReaderStrategy>(reader_strategy: &T) {
            let expected = include_bytes!("../Cargo.toml");

            let mut ffr = FastFile::read("Cargo.toml")
                .expect("Failed to create FastFileReaderBuilder")
                .open_with_strategy(reader_strategy)
                .expect("Failed to open path as FastFile");

            let mut bytes = Vec::new();
            ffr.read_to_end(&mut bytes)
                .expect("Failed to read from FastFile");

            asserting("File has been correctly read")
                .that(&bytes.as_slice())
                .is_equal_to(expected.as_ref());
        }
    }

    mod fast_read {
        use super::*;

        use crate::fastfile::FastFileRead;

        #[test]
        fn fastfilereader_read_correctly_with_file_backend() {
            let reader_strategy = TestFileReaderStragegy {};

            fastfilereader_read_correctly_tester(&reader_strategy);
        }

        #[test]
        fn fastfilereader_read_correctly_with_mmap_backend() {
            let reader_strategy = TestMmapReaderStragegy {};

            fastfilereader_read_correctly_tester(&reader_strategy);
        }

        fn fastfilereader_read_correctly_tester<T: strategy::ReaderStrategy>(reader_strategy: &T) {
            let expected = include_bytes!("../Cargo.toml");

            let mut ffr = FastFile::read("Cargo.toml")
                .expect("Failed to create FastFileReaderBuilder")
                .open_with_strategy(reader_strategy)
                .expect("Failed to open path as FastFile");

            let bytes = ffr.read_to_end().expect("Failed to read from FastFile");

            asserting("File has been correctly read")
                .that(&bytes)
                .is_equal_to(expected.as_ref());
        }
    }

    struct TestFileReaderStragegy {}
    impl strategy::ReaderStrategy for TestFileReaderStragegy {
        fn get_reader(&self, ffrb: FastFileReaderBuilder) -> Result<FastFileReader> {
            let FastFileReaderBuilder { file, size } = ffrb;
            let inner = BackingReader::file(file.unwrap())?;
            let size = size.unwrap_or(0);

            Ok(FastFileReader::new(inner, size))
        }
    }

    struct TestMmapReaderStragegy {}
    impl strategy::ReaderStrategy for TestMmapReaderStragegy {
        fn get_reader(&self, ffrb: FastFileReaderBuilder) -> Result<FastFileReader> {
            let FastFileReaderBuilder { file, size } = ffrb;
            let inner = BackingReader::mmap(file.unwrap())?;
            let size = size.unwrap_or(0);

            Ok(FastFileReader::new(inner, size))
        }
    }
}

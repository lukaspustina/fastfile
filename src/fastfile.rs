use crate::{errors::*, os};

use failure::Fail;
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

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
    file: Option<File>,
    size: Option<u64>,
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

    pub fn open(self) -> Result<FastFileReader> {
        let file = self.file.unwrap(); // Safe, since no code path allows to build w/o Some(file)
        let size = if let Some(size) = self.size {
            size
        } else {
            let meta = file
                .metadata()
                .map_err(|e| e.context(ErrorKind::FileOpFailed))?;
            meta.len()
        };

        let ff = FastFileReader { inner: file, size };

        ff.prepare_file_for_reading()?;

        Ok(ff)
    }
}

/// `FastFileReader` is a readable (`std::io::Read`) FastFile
pub struct FastFileReader {
    inner: File,
    size: u64,
}

impl FastFileReader {
    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn optimal_buffer_size(&self) -> usize {
        MAX_READ_BUF_SIZE.min(self.size.next_power_of_two() as usize)
    }

    fn prepare_file_for_reading(&self) -> Result<()> {
        os::prepare_file_for_reading(self)
    }
}

impl Read for FastFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

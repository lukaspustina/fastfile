use crate::{
    errors::*,
    fastfile::{BackingReader, FastFileReader, FastFileReaderBuilder},
    strategy::ReaderStrategy,
};

use failure::Fail;
use std::{fs::File, os::unix::io::AsRawFd};

pub struct DefaultMacOsReaderStrategy {}

impl ReaderStrategy for DefaultMacOsReaderStrategy {
    fn get_reader(&self, ffrb: FastFileReaderBuilder) -> Result<FastFileReader> {
        let size = get_file_size(&ffrb)?;
        let file = ffrb.file.unwrap(); // Safe, since no code path allows to build w/o Some(file)
        let inner = create_backing_reader(file, size)?;

        Ok(FastFileReader::new(inner, size))
    }
}

fn get_file_size(ffrb: &FastFileReaderBuilder) -> Result<u64> {
    let file = ffrb.file.as_ref().unwrap(); // Safe, since no code path allows to build w/o Some(file)
    let size = if let Some(size) = ffrb.size {
        size
    } else {
        let meta = file
            .metadata()
            .map_err(|e| e.context(ErrorKind::FileOpFailed))?;
        meta.len()
    };

    Ok(size)
}

fn create_backing_reader(file: File, file_size: u64) -> Result<BackingReader> {
    prepare_file_for_reading(&file, file_size)?;

    if file_size == 0 {
        BackingReader::mmap(file)
    } else {
        BackingReader::file(file)
    }
}

fn prepare_file_for_reading<T: AsRawFd>(fd: &T, _file_size: u64) -> Result<()> {
    let _fd = fd.as_raw_fd();

    Ok(())
}

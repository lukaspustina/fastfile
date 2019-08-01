use crate::{
    errors::*,
    fastfile::{BackingReader, FastFileReader, FastFileReaderBuilder},
    os::{read_advice, read_rdahead},
    strategy::ReaderStrategy,
};

use failure::Fail;
use std::{
    fs::File,
    os::unix::io::{AsRawFd, RawFd},
};

pub struct DefaultMacOsReaderStrategy {}

impl ReaderStrategy for DefaultMacOsReaderStrategy {
    fn get_reader(&self, ffrb: FastFileReaderBuilder) -> Result<FastFileReader> {
        let size = get_file_size(&ffrb)?;
        let file = ffrb.file.unwrap(); // Safe, since no code path allows to build w/o Some(file)

        let _ = prepare_file_for_reading(&file, size)?;
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
    if file_size < 0 {
        BackingReader::mmap(file)
    } else {
        BackingReader::file(file)
    }
}

fn prepare_file_for_reading<T: AsRawFd>(fd: &T, file_size: u64) -> Result<()> {
    let fd = fd.as_raw_fd();

    Ok(())
}

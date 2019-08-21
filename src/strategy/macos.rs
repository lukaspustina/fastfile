use crate::{
    errors::*,
    fastfile::{BackingReader, FastFileReader, FastFileReaderBuilder},
    os,
    strategy::ReaderStrategy,
};

use failure::Fail;
use std::{fs::File, os::unix::io::AsRawFd};

pub struct DefaultMacOsReaderStrategy {}

impl ReaderStrategy for DefaultMacOsReaderStrategy {
    fn get_reader(&self, ffrb: FastFileReaderBuilder) -> Result<FastFileReader> {
        let size = get_file_size(&ffrb)?;
        let file = ffrb.file;
        let inner = create_backing_reader(file, size)?;

        Ok(FastFileReader::new(inner, size))
    }
}

fn get_file_size(ffrb: &FastFileReaderBuilder) -> Result<usize> {
    let size = if let Some(size) = ffrb.size {
        size
    } else if let Some(size_hint) = ffrb.size_hint {
        size_hint
    } else {
        let file = &ffrb.file;
        let meta = file.metadata().map_err(|e| e.context(ErrorKind::FileOpFailed))?;
        meta.len() as usize
    };

    Ok(size)
}

fn create_backing_reader(file: File, file_size: usize) -> Result<BackingReader> {
    prepare_file_for_reading(&file, file_size)?;

    BackingReader::file(file)
}

#[allow(clippy::collapsible_if)]
fn prepare_file_for_reading<T: AsRawFd>(fd: &T, file_size: usize) -> Result<()> {

    if file_size >= 8 * 1024 {
        let fd = fd.as_raw_fd();
        if file_size <= 268_435_456 {
            os::read_ahead(fd)?;
        } else {
            os::read_advise(fd, file_size)?;
        }
    }

    Ok(())
}

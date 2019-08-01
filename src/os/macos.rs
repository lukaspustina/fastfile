use crate::{
    fastfile::{BackendReaderStragegy, BackendReaderStragegySelector, BackingReader},
    errors::*,
};

use failure::Fail;
use libc;
use memmap::Mmap;
use std::fs::File;
use std::os::unix::io::{AsRawFd, RawFd};

pub(crate) fn prepare_file_for_reading<T: AsRawFd>(fd: &T, file_size: u64) -> Result<()> {
    let fd = fd.as_raw_fd();

    Ok(())
}

fn read_advice() -> Result<()> {
    Ok(())
}

fn read_rdahead(fd: RawFd) -> Result<()> {
    let res = unsafe { libc::fcntl(fd, libc::F_RDAHEAD, 1) };
    if res < 0 {
        return Err(Error::from(ErrorKind::LibcFailed("fcntl F_RDAHEAD")))
                   .map_err(|e| e.context(ErrorKind::FileOpFailed).into());
    }

    Ok(())
}

pub struct MacOsBackendStrategySelector {
    file_size: u64,
}

impl MacOsBackendStrategySelector {
    pub fn new(file_size: u64) -> MacOsBackendStrategySelector {
        MacOsBackendStrategySelector {
            file_size,
        }
    }
}
impl BackendReaderStragegySelector for MacOsBackendStrategySelector {
    fn select(&self) -> BackendReaderStragegy {
        return BackendReaderStragegy::File;
    }
}

pub(crate) fn create_backing_reader<T: BackendReaderStragegySelector>(selector: &T, file: File) -> Result<BackingReader> {
    let res = match selector.select() {
        BackendReaderStragegy::File => BackingReader::File(file),
        BackendReaderStragegy::Mmap => {
            let mmap = unsafe {
                Mmap::map(&file)
                    .map_err(|e| e.context(ErrorKind::FileOpFailed))?
            };
            BackingReader::Mmap(file, std::io::Cursor::new(mmap))
        },
    };

    Ok(res)
}

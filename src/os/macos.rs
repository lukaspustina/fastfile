use crate::errors::*;

use failure::Fail;
use libc;
use std::os::unix::io::RawFd;

pub fn read_advice() -> Result<()> {
    Ok(())
}

pub fn read_rdahead(fd: RawFd) -> Result<()> {
    let res = unsafe { libc::fcntl(fd, libc::F_RDAHEAD, 1) };
    if res < 0 {
        return Err(Error::from(ErrorKind::LibcFailed("fcntl F_RDAHEAD")))
            .map_err(|e| e.context(ErrorKind::FileOpFailed).into());
    }

    Ok(())
}

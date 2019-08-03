use crate::errors::*;

use failure::Fail;
use libc;
use std::os::unix::io::RawFd;

#[allow(dead_code)]
pub fn read_advise(fd: RawFd, file_size: u64) -> Result<()> {
    let count: libc::c_int = file_size.min(libc::c_int::max_value() as u64) as libc::c_int;

    let ra = libc::radvisory {
        ra_offset: 0 as libc::off_t,
        ra_count: count,
    };
    let res = unsafe { libc::fcntl(fd, libc::F_RDADVISE, &ra) };
    if res < 0 {
        return Err(Error::from(ErrorKind::LibcFailed("fcntl F_RDADVISE")))
            .map_err(|e| e.context(ErrorKind::FileOpFailed).into());
    }

    Ok(())
}

#[allow(dead_code)]
pub fn read_ahead(fd: RawFd) -> Result<()> {
    let res = unsafe { libc::fcntl(fd, libc::F_RDAHEAD, 1) };
    if res < 0 {
        return Err(Error::from(ErrorKind::LibcFailed("fcntl F_RDAHEAD")))
            .map_err(|e| e.context(ErrorKind::FileOpFailed).into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use spectral::prelude::*;

    use std::{fs::File, os::unix::io::AsRawFd};

    #[test]
    fn test_read_advise() {
        let f = get_file();
        let file_size = f
            .metadata()
            .expect("Could not get metadata of test file")
            .len();

        let res = read_advise(f.as_raw_fd(), file_size);
        asserting("Read advise").that(&res).is_ok();
    }

    #[test]
    fn test_read_ahead() {
        let f = get_file();

        let res = read_ahead(f.as_raw_fd());

        asserting("Read advise").that(&res).is_ok();
    }

    fn get_file() -> File {
        File::open("Cargo.toml").expect("Could not open test file")
    }
}

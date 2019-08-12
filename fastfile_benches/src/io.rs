use libc;
use std::{
    fs::File,
    io::self,
    os::unix::io::AsRawFd,
    path::Path,
};

pub fn purge_cache<T: AsRef<Path>>(path: T) -> io::Result<()> {
    let file = File::open(path)?;
    let file_size = file.metadata()?.len();
    let fd = file.as_raw_fd();
    unsafe {
        let mem = libc::mmap(
            std::ptr::null_mut(),
            file_size as libc::size_t,
            libc::PROT_READ,
            libc::MAP_SHARED,
            fd,
            0,
        );
        if mem == libc::MAP_FAILED {
            eprintln!("mmap failed");
            return Err(io::Error::from(io::ErrorKind::Other));
        }
        let res = libc::msync(mem, file_size as libc::size_t, libc::MS_INVALIDATE);
        if res < 0 {
            eprintln!("msync failed");
            return Err(io::Error::from(io::ErrorKind::Other));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use spectral::prelude::*;

    #[test]
    fn test_purge_cache() {
        let path = "Cargo.toml";

        let res = purge_cache(path);

        asserting("Page cache purged").that(&res).is_ok();
    }
}

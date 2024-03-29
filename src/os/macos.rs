use crate::{
    errors::*,
    os::{PageCacheInfo, PAGE_SIZE},
};

use failure::Fail;
use libc;
use std::os::unix::io::RawFd;

#[allow(dead_code)]
pub fn read_advise(fd: RawFd, file_size: usize) -> Result<()> {
    let count: libc::c_int = file_size.min(libc::c_int::max_value() as usize) as libc::c_int;

    let ra = libc::radvisory {
        ra_offset: 0 as libc::off_t,
        ra_count:  count,
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

#[allow(dead_code)]
pub fn get_page_cache_info(fd: RawFd, file_size: usize) -> Result<PageCacheInfo> {
    let mem = unsafe {
        let mem = libc::mmap(
            std::ptr::null_mut(),
            file_size as libc::size_t,
            libc::PROT_READ,
            libc::MAP_SHARED,
            fd,
            0,
        );
        if mem == libc::MAP_FAILED {
            return Err(Error::from(ErrorKind::LibcFailed("mmap")))
                .map_err(|e| e.context(ErrorKind::FileOpFailed).into());
        }
        mem
    };

    let num_pages = bytes_in_pages(file_size);
    let mut pages: Vec<libc::c_char> = Vec::with_capacity(num_pages);
    unsafe {
        pages.set_len(num_pages);
    }

    let pages_array = pages.as_mut_slice();
    unsafe {
        let res = libc::mincore(mem, file_size as libc::size_t, pages_array.as_mut_ptr());
        if res < 0 {
            return Err(Error::from(ErrorKind::LibcFailed("mincore")))
                .map_err(|e| e.context(ErrorKind::FileOpFailed).into());
        }
    }
    let num_cached_pages = pages_array.iter().map(|x| (x & 0x1) as usize).sum();

    let pci = PageCacheInfo {
        total:  num_pages,
        cached: num_cached_pages,
    };

    Ok(pci)
}

fn bytes_in_pages(bytes: usize) -> usize { ((bytes + PAGE_SIZE - 1) / PAGE_SIZE) }

#[cfg(test)]
mod tests {
    use super::*;

    use spectral::prelude::*;

    use std::{fs::File, os::unix::io::AsRawFd};

    #[test]
    fn test_read_advise() {
        let f = get_file();
        let file_size = f.metadata().expect("Could not get metadata of test file").len() as usize;

        let res = read_advise(f.as_raw_fd(), file_size);
        asserting("Read advise").that(&res).is_ok();
    }

    #[test]
    fn test_read_ahead() {
        let f = get_file();

        let res = read_ahead(f.as_raw_fd());

        asserting("Read advise").that(&res).is_ok();
    }

    #[test]
    fn test_get_page_cache_info() {
        let f = get_file();
        let file_size = f.metadata().expect("Could not get metadata of test file").len() as usize;

        let res = get_page_cache_info(f.as_raw_fd(), file_size);
        asserting("Get page cache information").that(&res.is_ok()).is_true();

        let pci = res.unwrap();
        asserting("Number of pages").that(&pci.total()).is_equal_to(&1);
        // Cargo.toml is always cached due to `cargo test` obviously reads it.
        asserting("Number of cached pages").that(&pci.cached()).is_equal_to(&1);
        asserting("Cached ratio").that(&pci.ratio()).is_equal_to(&1f32);
    }

    fn get_file() -> File { File::open("Cargo.toml").expect("Could not open test file") }
}

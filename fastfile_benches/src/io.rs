use libc;
use ring::digest::{Context, Digest, SHA256};
use std::io::{self, Write};
use std::fs::File;
use std::path::Path;
use std::os::unix::io::AsRawFd;

pub const MAX_READ_BUF_SIZE: usize = 64 * 1024;

pub struct WriterWithSha<'a, T: Write> {
    inner: &'a mut T,
    digest: Context,
}

impl<'a, T: Write> WriterWithSha<'a, T> {
    pub fn new(writer: &'a mut T) -> WriterWithSha<'a, T> {
        WriterWithSha {
            inner: writer,
            digest: Context::new(&SHA256),
        }
    }

    pub fn digest(self) -> Digest {
        self.digest.finish()
    }
}

impl<'a, T: Write> Write for WriterWithSha<'a, T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.digest.update(buf);
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

pub struct ConsumingSink {
    inner: [u8; MAX_READ_BUF_SIZE],
    buf_size: usize,
    written: usize,
}

impl ConsumingSink {
    pub fn new() -> ConsumingSink {
        ConsumingSink {
            inner: [0u8; MAX_READ_BUF_SIZE],
            buf_size: MAX_READ_BUF_SIZE,
            written: 0,
        }
    }

    pub fn written(&self) -> usize {
        self.written
    }
}

impl Write for ConsumingSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let buf_len = buf.len();

        if buf_len == 0 {
            return Ok(0);
        }

        let mut written = 0;
        for chunk in buf.chunks(self.buf_size) {
            let chunk_len = chunk.len();
            self.inner[0..chunk_len].copy_from_slice(chunk);
            written += chunk_len;
        }
        assert_eq!(written, buf.len());

        self.written += written;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn purge_cache<T: AsRef<Path>>(path: T) -> io::Result<()> {
    let file = File::open(path)?;
    let file_size = file.metadata()?.len();
    let fd = file.as_raw_fd();
    unsafe {
        let mem = libc::mmap(std::ptr::null_mut(), file_size as libc::size_t, libc::PROT_READ, libc::MAP_SHARED, fd, 0);
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

    #[test]
    fn conuming_sink_empty_src() {
        let buf = vec![0u8; 0];

        let mut sink = ConsumingSink::new();
        let res = sink.write(&buf);

        assert_that(&res).is_ok().is_equal_to(&0);
    }

    #[test]
    fn conuming_sink_src_smaller_buf() {
        let buf = vec![0u8; 123];

        let mut sink = ConsumingSink::new();
        let res = sink.write(&buf);

        assert_that(&res).is_ok().is_equal_to(&buf.len());
    }

    #[test]
    fn conuming_sink_src_exact_buf() {
        let buf = vec![0u8; 64*1024];

        let mut sink = ConsumingSink::new();
        let res = sink.write(&buf);

        assert_that(&res).is_ok().is_equal_to(&buf.len());
    }

    #[test]
    fn conuming_sink_src_larger_buf() {
        let buf = vec![0u8; 64*1024 + 1];

        let mut sink = ConsumingSink::new();
        let res = sink.write(&buf);

        assert_that(&res).is_ok().is_equal_to(&buf.len());
    }
}

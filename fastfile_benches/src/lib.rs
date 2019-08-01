use fastfile::prelude::*;

use ring::digest::{Context, Digest, SHA256};
use std::io::{self, Write};

pub static FILE_SIZES: &[usize] = &[
    1024,
    4 * 1024,
    16 * 1024,
    256 * 1024,
    1024 * 1024,
    10 * 1024 * 1024,
    100 * 1024 * 1024,
];

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

#[cfg(test)]
mod tests {
    use super::*;

    use spectral::prelude::*;

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
        let buf = vec![0u8; MAX_READ_BUF_SIZE];

        let mut sink = ConsumingSink::new();
        let res = sink.write(&buf);

        assert_that(&res).is_ok().is_equal_to(&buf.len());
    }

    #[test]
    fn conuming_sink_src_larger_buf() {
        let buf = vec![0u8; MAX_READ_BUF_SIZE + 1];

        let mut sink = ConsumingSink::new();
        let res = sink.write(&buf);

        assert_that(&res).is_ok().is_equal_to(&buf.len());
    }
}

pub mod benches {
    use rand::{rngs::SmallRng, Rng, SeedableRng};
    use std::{
        fmt,
        fs::{self, File},
        io::{self, Write},
        path::{Path, PathBuf},
    };
    use tempfile::NamedTempFile;

    pub struct Param {
        pub path: PathBuf,
        pub size: usize,
    }

    impl Param {
        pub fn new(path: PathBuf, size: usize) -> Param {
            Param { path, size }
        }
    }

    impl fmt::Debug for Param {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "file size: {}", self.size)
        }
    }

    pub fn setup(file_sizes: &[usize]) -> io::Result<Vec<Param>> {
        let mut small_rng = SmallRng::from_entropy();
        let mut params = Vec::new();
        for size in file_sizes {
            let path = {
                let file = NamedTempFile::new()?;
                file.path().to_path_buf()
            };
            fill_file(&path, *size, &mut small_rng)?;
            params.push(Param::new(path, *size));
        }

        Ok(params)
    }

    pub fn teardown<P: AsRef<Path>>(paths: &[P]) {
        for p in paths {
            let _ = fs::remove_file(p);
        }
    }

    fn fill_file<P: AsRef<Path>, R: Rng>(path: P, size: usize, rng: &mut R) -> io::Result<usize> {
        assert!(
            size / 1024 > 0 && size % 1024 == 0,
            "fill_file currently only supports 1KB chunks"
        );
        let mut file = File::create(path)?;

        let mut buf = [0u8; 1024];
        rng.try_fill(&mut buf[..])
            .expect("failed to generate rnd buf");
        for _ in 0..(size / 1024) {
            file.write_all(&buf)?;
        }

        Ok(size)
    }

}

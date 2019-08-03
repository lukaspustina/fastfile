use fastfile::fastfile::MAX_READ_BUF_SIZE;
use ring::digest::{Context, Digest, SHA256};
use std::io::{self, Write};

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
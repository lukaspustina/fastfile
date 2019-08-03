pub mod fastfile {
    pub mod fastread {
        use fastfile::prelude::*;
        use fastfile::FastFileRead;
        use std::{
            io::{self},
            path::Path,
        };

        pub fn read<P: AsRef<Path>>(path: P) -> io::Result<(u64, u64)> {
            let mut ffr = FastFile::read(path)
                .expect("Failed to create FastFileReaderBuilder")
                .open()
                .expect("Failed to open path as FastFile");

            let mut sum = 064;
            let mut bytes_read = 0u64;
            loop {
                let len = match ffr.read() {
                    Ok(buf) if buf.is_empty() => return Ok((bytes_read, sum)),
                    Ok(buf) => {
                        sum += buf.iter().map(|x| *x as u64).sum::<u64>();
                        buf.len()
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                };
                bytes_read += len as u64;
            }
        }
    }

    pub mod read {
        use fastfile::prelude::*;
        use std::{
            io::{self, Read},
            path::{Path},
        };

        pub fn read<P: AsRef<Path>>(path: P) -> io::Result<(u64, u64)> {
            let mut ffr = FastFile::read(path)
                .expect("Failed to create FastFileReaderBuilder")
                .open()
                .expect("Failed to open path as FastFile");

            let mut buf = prepare_buf!(ffr);
            let mut sum = 064;
            let mut bytes_read = 0u64;
            loop {
                let len = match ffr.read(&mut buf[0..MAX_READ_BUF_SIZE]) {
                    Ok(0) => return Ok((bytes_read, sum)),
                    Ok(len) => len,
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                };
                bytes_read += len as u64;

                sum += buf.iter().map(|x| *x as u64).sum::<u64>();
            }
        }
    }
}

pub mod stdlib {
    pub mod buf_read {
        use fastfile::fastfile::MAX_READ_BUF_SIZE;
        use std::{
            io::{self, BufReader, Read},
            fs::File,
            path::{Path},
        };

        pub fn read<P: AsRef<Path>>(path: P) -> io::Result<(u64, u64)> {
            let file = File::open(path)
                .expect("Failed to open path as File");
            let mut reader = BufReader::new(file);

            let mut buf = [0u8; MAX_READ_BUF_SIZE];
            let mut sum = 064;
            let mut bytes_read = 0u64;
            loop {
                let len = match reader.read(&mut buf[0..MAX_READ_BUF_SIZE]) {
                    Ok(0) => return Ok((bytes_read, sum)),
                    Ok(len) => len,
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                };
                bytes_read += len as u64;

                sum += buf.iter().map(|x| *x as u64).sum::<u64>();
            }
        }
    }
}
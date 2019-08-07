pub mod fastfile {
    pub mod fastread {
        use fastfile::{prelude::*, FastFileRead};
        use std::{io, path::Path};

        pub fn read<P: AsRef<Path>>(path: P) -> io::Result<(u64, u64)> {
            let path = path.as_ref();
            let mut ffr = FastFile::read(path)
                .expect("Failed to create FastFileReaderBuilder")
                .open()
                .expect("Failed to open path as FastFile");

            let mut sum = 0u64;
            let mut bytes_read = 0u64;
            loop {
                let len = match ffr.read() {
                    Ok(buf) if buf.is_empty() => return Ok((bytes_read, sum)),
                    Ok(buf) => {
                        sum += buf.iter().map(|x| u64::from(*x)).sum::<u64>();
                        buf.len()
                    }
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
            path::Path,
        };

        pub fn read<P: AsRef<Path>>(path: P) -> io::Result<(u64, u64)> {
            let path = path.as_ref();
            let mut ffr = FastFile::read(path)
                .expect("Failed to create FastFileReaderBuilder")
                .open()
                .expect("Failed to open path as FastFile");

            let mut buf = prepare_buf!(ffr, 65_536);
            let mut sum = 0u64;
            let mut bytes_read = 0u64;
            loop {
                let len = match ffr.read(&mut buf[..]) {
                    Ok(0) => return Ok((bytes_read, sum)),
                    Ok(len) => len,
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                };
                bytes_read += len as u64;

                sum += buf.iter().map(|x| u64::from(*x)).sum::<u64>();
            }
        }
    }
}

pub mod stdlib {
    pub mod buf_read {
        use std::{
            fs::File,
            io::{self, BufReader, Read},
            path::Path,
        };

        pub fn read<P: AsRef<Path>>(path: P) -> io::Result<(u64, u64)> {
            let path = path.as_ref();
            let file = File::open(path).expect("Failed to open path as File");
            let mut reader = BufReader::new(file);

            let mut buf = [0u8; 8 * 1024];
            let mut sum = 0u64;
            let mut bytes_read = 0u64;
            loop {
                let len = match reader.read(&mut buf[..]) {
                    Ok(0) => return Ok((bytes_read, sum)),
                    Ok(len) => len,
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                };
                bytes_read += len as u64;

                sum += buf.iter().map(|x| u64::from(*x)).sum::<u64>();
            }
        }
    }
}

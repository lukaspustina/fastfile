// TODO: move to stable with mem::MaybeUninit::uninit()
#![feature(read_initializer)]

use fastfile::prelude::*;

use std::io::{self, Read};

fn main() {
    let path = "Cargo.toml";

    let mut ffr = FastFile::read(path)
        .expect("Failed to create FastFileReaderBuilder")
        .open()
        .expect("Failed to open path as FastFile");
    let buf_size = ffr.optimal_buffer_size();
    let bytes_read = read(&mut ffr, buf_size).expect("Failed to read file");

    assert_eq!(bytes_read, ffr.size(), "Read bytes differ from file size");
    println!("Bytes read: {}, expected: {}", bytes_read, ffr.size());
}

fn read<R: Read + Sized>(reader: &mut R, buf_size: usize) -> std::io::Result<u64> {
    let mut buf = prepare_buf!(reader);

    let mut bytes_read = 0u64;
    loop {
        let len = match reader.read(&mut buf[0..buf_size]) {
            Ok(0) => return Ok(bytes_read),
            Ok(len) => len,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        bytes_read += len as u64;
    }
}

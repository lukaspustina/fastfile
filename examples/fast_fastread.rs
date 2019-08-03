use fastfile::prelude::*;
use fastfile::FastFileRead;

fn main() {
    let path = "Cargo.toml";

    let mut ffr = FastFile::read(path)
        .expect("Failed to create FastFileReaderBuilder")
        .open()
        .expect("Failed to open path as FastFile");
    let buf = ffr.read().expect("Failed to fastread file");
    let len: u64 = buf.len() as u64;

    assert_eq!(len, ffr.size(), "Read bytes differ from file size");
    println!("Bytes read: {}, expected: {}", len, ffr.size());
}

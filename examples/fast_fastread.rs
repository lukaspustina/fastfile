use fastfile::{prelude::*, FastFileRead};
use std::env;

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "Cargo.toml".to_string());

    let mut ffr = FastFile::read(&path)
        .expect("Failed to create FastFileReaderBuilder")
        .open()
        .expect("Failed to open path as FastFile");

    let mut len = 0u64;
    loop {
        let buf = ffr.read().expect("Failed to fastread file");
        if buf.is_empty() {
            break;
        };
        len += buf.len() as u64;
    }

    assert_eq!(len, ffr.size(), "Read bytes differ from file size");
    println!("Bytes read: {}, expected: {}", len, ffr.size());
}

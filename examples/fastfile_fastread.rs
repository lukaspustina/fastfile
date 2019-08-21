use fastfile::{prelude::*, FastFileRead};
use std::env;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| "Cargo.toml".to_string());

    let mut ffr = FastFile::read(&path)
        .expect("Failed to create FastFileReaderBuilder")
        .open()
        .expect("Failed to open path as FastFile");

    let mut bytes_read = 0usize;
    let mut sum = 0usize;
    loop {
        let buf = ffr.read().expect("Failed to fastread file");
        if buf.is_empty() {
            break;
        };
        bytes_read += buf.len();
        sum += buf.iter().map(|x| usize::from(*x)).sum::<usize>();
    }
    assert_eq!(bytes_read, ffr.size(), "Read bytes differ from file size");
    println!("Bytes read: {}, expected: {}, sum: {}", bytes_read, ffr.size(), sum);
}

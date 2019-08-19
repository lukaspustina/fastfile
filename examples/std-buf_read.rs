use std::{
    env,
    io::{self, BufReader, Read},
    fs::File,
};

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| "Cargo.toml".to_string());

    let file = File::open(path).expect("Failed to open path as File");
    let mut reader = BufReader::new(file);

    let bytes_read = read(&mut reader).expect("Failed to read file");

    println!("Bytes read: {}, sum: {}", bytes_read.0, bytes_read.1);
}

fn read<R: Read + Sized>(reader: &mut R) -> std::io::Result<(u64, u64)> {
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

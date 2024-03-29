use std::{
    env,
    fs::File,
    io::{self, BufReader, Read},
};

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| "Cargo.toml".to_string());

    let file = File::open(path).expect("Failed to open path as File");
    let mut reader = BufReader::new(file);

    let bytes_read = read(&mut reader).expect("Failed to read file");

    println!("Bytes read: {}, sum: {}", bytes_read.0, bytes_read.1);
}

fn read<R: Read + Sized>(reader: &mut R) -> std::io::Result<(usize, usize)> {
    let mut buf = [0u8; 8 * 1024];
    let mut sum = 0usize;
    let mut bytes_read = 0usize;

    loop {
        let len = match reader.read(&mut buf[..]) {
            Ok(0) => return Ok((bytes_read, sum)),
            Ok(len) => len,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        bytes_read += len;

        sum += buf.iter().map(|x| usize::from(*x)).sum::<usize>();
    }
}

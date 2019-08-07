use ring::digest::{Context, Digest, SHA256};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use tempfile::NamedTempFile;
use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    path::{Path, PathBuf},
};

pub fn create_random_test_file(size: usize) -> io::Result<PathBuf> {
    let path = {
        let file = NamedTempFile::new()?;
        file.path().to_path_buf()
    };
    fill_file(&path, size)?;

    {
        let file = File::open(&path)?;
        let file_size = file.metadata()?.len();
        assert_eq!(file_size, size as u64, "Failed to fill file with random data");
    }

    Ok(path)
}

pub fn fill_file<P: AsRef<Path>>(path: P, size: usize) -> io::Result<usize> {
    use fastfile::os::PAGE_SIZE;

    let mut file = File::create(path)?;
    let mut rng = SmallRng::from_entropy();

    let mut buf = [0u8; PAGE_SIZE];
    rng.try_fill(&mut buf[..])
        .expect("failed to generate rnd buf");

    for _ in 0..(size / PAGE_SIZE) {
        file.write_all(&buf)?;
    }
    file.write_all(&buf[0..size % PAGE_SIZE])?;

    file.sync_all()?;

    Ok(size)
}

pub fn get_digest_for_path<P: AsRef<Path>>(path: P) -> io::Result<Digest> {
    let file = File::open(path).expect("Failed to open path as File");
    let mut reader = BufReader::new(file);
    let mut digest = Context::new(&SHA256);

    let mut buf = [0u8; 8 * 1024];
    loop {
        let len = match reader.read(&mut buf[..]) {
            Ok(0) => break,
            Ok(len) => len,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        digest.update(&buf[0..len]);
    }

    Ok(digest.finish())
}


use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::{
    fmt,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

pub mod methods;

pub struct Param {
    pub path: PathBuf,
    pub size: usize,
}

impl Param {
    pub fn new(path: PathBuf, size: usize) -> Param {
        Param { path, size }
    }
}

impl fmt::Debug for Param {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "file size: {}", self.size)
    }
}

pub fn setup(file_sizes: &[usize]) -> io::Result<Vec<Param>> {
    let mut small_rng = SmallRng::from_entropy();
    let mut params = Vec::new();
    for size in file_sizes {
        let path = {
            let file = NamedTempFile::new()?;
            file.path().to_path_buf()
        };
        fill_file(&path, *size, &mut small_rng)?;
        params.push(Param::new(path, *size));
    }

    Ok(params)
}

pub fn teardown<P: AsRef<Path>>(paths: &[P]) {
    for p in paths {
        let _ = fs::remove_file(p);
    }
}

fn fill_file<P: AsRef<Path>, R: Rng>(path: P, size: usize, rng: &mut R) -> io::Result<usize> {
    assert!(
        size / 1024 > 0 && size % 1024 == 0,
        "fill_file currently only supports 1KB chunks"
    );
    let mut file = File::create(path)?;

    let mut buf = [0u8; 1024];
    rng.try_fill(&mut buf[..])
        .expect("failed to generate rnd buf");
    for _ in 0..(size / 1024) {
        file.write_all(&buf)?;
    }

    Ok(size)
}

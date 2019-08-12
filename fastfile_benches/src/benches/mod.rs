use crate::utils;

use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

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
    let mut params = Vec::new();
    for &size in file_sizes {
        let path = utils::create_random_test_file(size)?;
        params.push(Param::new(path, size));
    }

    Ok(params)
}

pub fn teardown<P: AsRef<Path>>(paths: &[P]) {
    for p in paths {
        let _ = fs::remove_file(p);
    }
}

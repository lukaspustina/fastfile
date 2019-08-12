use fastfile_benches::FILE_SIZES;
use fastfile_benches::benches::*;
use fastfile_benches::utils::create_random_test_file;

use fastfile::prelude::*;
use std::{
    env,
    io,
    fs::{self, File},
    path::{Path, PathBuf},
};

fn fastread<P: AsRef<Path>>(path: &P) {
    use fastfile::FastFileRead;

    let mut ffr = FastFile::read(path)
        .expect("Failed to create FastFileReaderBuilder")
        .open()
        .expect("Failed to open path as FastFile");

    loop {
        let buf = ffr.read().expect("Failed to fastread file");
        if buf.is_empty() {
            break;
        };
    }
}

fn main() {
    let num: usize = env::args()
        .nth(1)
        .map(|x| x.parse::<usize>().unwrap())
        .unwrap_or_else(|| 5usize);
    let output_file: Option<String> = env::args().nth(2);

    let params = prepare(FILE_SIZES)
        .expect("Failed to create test files");

    let benchmark = Benchmark::new("FastFile read", &params, num)
        .add_func("fastread", fastread);

    let res = benchmark.benchmark();

    if let Some(ref path) = output_file {
        write_csv(path, &res)
            .expect("Failed write output file");
    }

    cleanup(params)
        .expect("Failed to clean up test files");
}

fn prepare(file_sizes: &[usize]) -> io::Result<Vec<Param<PathBuf>>> {
    let mut params = Vec::with_capacity(file_sizes.len());

    for size in file_sizes {
        let path = create_random_test_file(1024)?;
        let param_str = format!("{}", size);
        let p = Param::new(param_str, path);
        params.push(p);
    }

    Ok(params)
}

fn write_csv<P: AsRef<Path>>(path: P, res: &BenchmarkResult) -> io::Result<()> {
    let mut file = File::create(path)?;
    res.write_as_csv(&mut file) 
}

fn cleanup(params: Vec<Param<PathBuf>>) -> io::Result<()> {
    for p in params {
        fs::remove_file(p.value())?;
    }

    Ok(())
}


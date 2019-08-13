use fastfile_benches::FILE_SIZES;
use fastfile_benches::benches::*;
use fastfile_benches::utils::create_random_test_file;

use byte_unit::Byte;
use fastfile::prelude::*;
use std::{
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

fn purge_cache<P: AsRef<Path>>(path: &P) {
    let _ = fastfile_benches::io::purge_cache(path);
}

fn main() {
    let results_dir = "./results/current";
    let benchmark_name = "FastFile Read (NOT cached)";
    let iterations = 1000;
    let params = prepare(&FILE_SIZES[0..10])
        .expect("Failed to create test files");

    let benchmark = Benchmark::new(benchmark_name, &params, iterations)
        .setup(purge_cache)
        .add_func("fastread", fastread);

    let res = benchmark.benchmark();

    fs::create_dir_all(results_dir)
        .expect("Failed create results directory");
    let output_path = format!("{}/{}.csv", results_dir, benchmark_name);
    write_csv(output_path, &res)
        .expect("Failed write output file");

    cleanup(params)
        .expect("Failed to clean up test files");
}

fn prepare(file_sizes: &[usize]) -> io::Result<Vec<Param<PathBuf>>> {
    let mut params = Vec::with_capacity(file_sizes.len());

    for &size in file_sizes {
        let name = format!("{}", size);
        let bytes = Byte::from_bytes(size as u128);
        let display_name= format!("{}", bytes.get_appropriate_unit(true).format(0));
        let path = create_random_test_file(size)?;
        let p = Param::new(name, display_name, size, path);
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


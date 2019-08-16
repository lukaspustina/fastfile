use fastfile_benches::FILE_SIZES_VERY_SMALL;
use fastfile_benches::benches::*;
use fastfile_benches::benches::methods::fastfile::fastread;
use fastfile_benches::utils::create_random_test_file;

use byte_unit::Byte;
use std::{
    io,
    fs::{self, File},
    path::{Path, PathBuf},
};

fn main() {
    let results_dir = "./results/current";
    let benchmark_name = "FastFile: fastread, NOT cached, very small [1 KiB - 128 KiB]";
    let iterations = 10000;
    let params = prepare(&FILE_SIZES_VERY_SMALL).expect("Failed to create test files");

    let benchmark = Benchmark::new(benchmark_name, &params, iterations)
        .setup(|p| { let _ = fastfile_benches::io::purge_cache(p); })
        .add_func("fastread", |p| { let _ = fastread::read(p); });

    let res = benchmark.benchmark();
    write_results(&res, benchmark_name, results_dir).expect("Failed write results file");
    cleanup(params).expect("Failed to clean up test files");
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

fn write_results<P: AsRef<Path>>(results: &BenchmarkResult, benchmark_name: &str, results_dir: P) -> io::Result<()> {
    fs::create_dir_all(&results_dir)?;
    let mut output_path = results_dir.as_ref().join(benchmark_name);
    output_path.set_extension("csv");
    println!("Writing benchmark results to \"{}\"", &output_path.to_string_lossy());
    write_csv(output_path, results)?;

    Ok(())
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


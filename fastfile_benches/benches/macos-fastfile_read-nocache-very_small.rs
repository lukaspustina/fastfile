use fastfile_benches::{
    benches::{cleanup, methods::fastfile::read::read, prepare, FILE_SIZES_VERY_SMALL},
    benchmark::Benchmark,
};

fn main() {
    let benchmark_name = "FastFile: read, NOT cached, very small [1 KiB - 128 KiB]";
    let iterations = 10000;
    let params = prepare(&FILE_SIZES_VERY_SMALL).expect("Failed to create test files");

    let benchmark = Benchmark::new(benchmark_name, &params, iterations)
        .setup(|p| {
            let _ = fastfile_benches::io::purge_cache(p);
        })
        .add_func("fastread", |p| {
            let _ = read(p, Some(*FILE_SIZES_VERY_SMALL.last().unwrap())); // Safe, because slice is not empty
        });

    benchmark
        .benchmark()
        .write_results("./results/current")
        .expect("Failed to write benchmark results");
    cleanup(params).expect("Failed to clean up test files");
}

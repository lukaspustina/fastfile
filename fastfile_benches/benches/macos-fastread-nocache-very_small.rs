use fastfile_benches::{
    benches::{cleanup, methods::fastfile::fastread, prepare, FILE_SIZES_VERY_SMALL},
    benchmark::Benchmark,
};

fn main() {
    let benchmark_name = "FastFile: fastread, NOT cached, very small [1 KiB - 128 KiB]";
    let iterations = 10000;
    let params = prepare(&FILE_SIZES_VERY_SMALL).expect("Failed to create test files");

    let benchmark = Benchmark::new(benchmark_name, &params, iterations)
        .setup(|p| {
            let _ = fastfile_benches::io::purge_cache(p);
        })
        .add_func("fastread", |p| {
            fastread::read(p)
        });

    benchmark
        .benchmark()
        .write_results("./results/current")
        .expect("Failed to write benchmark results");
    cleanup(params).expect("Failed to clean up test files");
}

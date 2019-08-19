use fastfile_benches::{
    benches::{cleanup, methods::std::buf_read::read, prepare, FILE_SIZES_VERY_SMALL},
    benchmark::Benchmark,
};

#[allow(clippy::redundant_closure)] // this is for clippy and the read closure
fn main() {
    let benchmark_name = "Std: buf_read, NOT cached, very small [1 KiB - 128 KiB]";
    let iterations = 10000;
    let params = prepare(&FILE_SIZES_VERY_SMALL).expect("Failed to create test files");

    let benchmark = Benchmark::new(benchmark_name, &params, iterations)
        .setup(|p| {
            let _ = fastfile_benches::io::purge_cache(p);
        })
        .add_func("fastread", |p| read(p));

    benchmark
        .benchmark()
        .write_results("./results/current")
        .expect("Failed to write benchmark results");
    cleanup(params).expect("Failed to clean up test files");
}

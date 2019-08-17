use fastfile_benches::{
    benches::{methods::std::buf_read, prepare, cleanup, FILE_SIZES_VERY_SMALL},
    benchmark::Benchmark,
};

fn main() {
    let benchmark_name = "Std: buf_read, NOT cached, very small [1 KiB - 128 KiB]";
    let iterations = 10000;
    let params = prepare(&FILE_SIZES_VERY_SMALL).expect("Failed to create test files");

    let benchmark = Benchmark::new(benchmark_name, &params, iterations)
        .setup(|p| {
            let _ = fastfile_benches::io::purge_cache(p);
        })
        .add_func("fastread", |p| {
            let _ = buf_read::read(p);
        });

    benchmark.benchmark().write_results("./results/current").expect("Failed to write benchmark results");
    cleanup(params).expect("Failed to clean up test files");
}


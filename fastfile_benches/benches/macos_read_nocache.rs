#![feature(read_initializer)]

use fastfile_benches::{benches::*, *};

use criterion::*;
use std::path::PathBuf;

fn bench_impls(c: &mut Criterion) {
    let params = setup(&FILE_SIZES).expect("failed to generate test files");
    let paths: Vec<PathBuf> = params.iter().map(|x| x.path.clone()).collect();

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    c.bench(
        "fastfile macos (NOT cached)",
        ParameterizedBenchmark::new(
            "read",
            |b, param| b.iter_batched_ref(
                || crate::io::purge_cache(&param.path).expect("Failed to purge page cache for file"),
                |_| methods::fastfile::read::read(&param.path),
                BatchSize::NumIterations(1)
            ),
            params,
        )
        .throughput(|param| Throughput::Bytes(param.size as u32))
        .plot_config(plot_config),
    );

    teardown(&paths);
}

criterion_group!(name = benches; config = Criterion::default().sample_size(10); targets = bench_impls);
criterion_main!(benches);

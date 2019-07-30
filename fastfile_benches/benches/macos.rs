#![feature(read_initializer)]

use fastfile_benches::*;
use fastfile_benches::benches::*;

use criterion::*;
use fastfile::prelude::*;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

fn read<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    let mut ffr = FastFile::read(path)
        .expect("Failed to create FastFileReaderBuilder")
        .open()
        .expect("Failed to open path as FastFile");
    let buf_size = ffr.optimal_buffer_size();

    let mut buf = prepare_buf!(ffr);
    let mut bytes_read = 0u64;
    loop {
        let len = match ffr.read(&mut buf[0..buf_size]) {
            Ok(0) => return Ok(bytes_read),
            Ok(len) => len,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        bytes_read += len as u64;
    }
}

fn bench_impls(c: &mut Criterion) {
    let params = setup(&FILE_SIZES).expect("failed to generate test files");
    let paths: Vec<PathBuf> = params.iter().map(|x| x.path.clone()).collect();

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    c.bench(
        "fastfile",
        ParameterizedBenchmark::new(
            "read",
            |b, param| b.iter(|| read(&param.path)),
            params,
        )
        .throughput(|param| Throughput::Bytes(param.size as u32))
        .plot_config(plot_config),
    );

    teardown(&paths);
}

criterion_group!(name = benches; config = Criterion::default().sample_size(157); targets = bench_impls);
criterion_main!(benches);


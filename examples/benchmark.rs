use fastfile::prelude::*;
use fastfile_benches::utils::{create_random_test_file};
use std::{
    env,
    io::{self, Write},
    fs::File,
    path::Path,
    time::Instant,
};

trait AsCsv {
    fn write_as_csv<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}

#[derive(Debug)]
struct BenchmarkResult<'a> {
    pub file_size: usize,
    pub strategy: &'a str,
    pub samples: Vec<Sample<'a>>,
}

impl<'a> BenchmarkResult<'a> {
    fn new(num: usize, file_size: usize, strategy: &'a str) -> BenchmarkResult {
        let samples = Vec::with_capacity(num);
        BenchmarkResult {
            file_size,
            strategy,
            samples
        }
    }

    fn add(&mut self, sample: Sample<'a>) {
        self.samples.push(sample)
    }

    fn summary(&self) -> Summary {
        let mut min = u128::max_value();
        let mut max = u128::min_value();
        let mut sum = 0;

        for s in &self.samples {
            sum += s.time;
            min = min.min(s.time);
            max = max.max(s.time);
        }
        let mean = sum / self.samples.len() as u128;

        Summary::new(min, max, mean)
    }
}

impl<'a> AsCsv for BenchmarkResult<'a> {
    fn write_as_csv<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "strategy,file_size,time")?;
        for s in &self.samples {
            writeln!(writer, "{},{},{}", s.strategy, s.file_size, s.time)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Sample<'a> {
    pub strategy: &'a str,
    pub file_size: usize,
    pub time: u128,
}

impl<'a> Sample<'a> {
    fn new(strategy: &'a str, file_size: usize, time: u128) -> Sample {
        Sample { strategy, file_size, time }
    }
}

#[derive(Debug)]
struct Summary {
    pub min: u128,
    pub max: u128,
    pub mean: u128,
}

impl Summary {
   fn new(min: u128, max: u128, mean: u128) -> Summary {
        Summary {
            min, max, mean,
        }
    }
}

fn benchmark<'a, F: Fn(&Path) -> ()>(num: usize, size: usize, reader: F, name: &'a str) -> io::Result<BenchmarkResult<'a>> {
    let mut res = BenchmarkResult::new(num, size, name);

    // setup
    let test_file = create_random_test_file(size)?;

    for _ in 0..num {
        // pre
        fastfile_benches::io::purge_cache(&test_file)?;

        // measure
        let time = measure(
            || reader(&test_file.as_path())
        );
        res.add(Sample::new(&res.strategy, res.file_size, time));

        // post
    }

    // teardown

    Ok(res)
}

fn measure<O, F: Fn() -> O>(func: F) -> u128 {
    let start = Instant::now();
    func();
    start.elapsed().as_nanos()
}

fn fastread(path: &Path) {
    use fastfile::FastFileRead;

    let mut ffr = FastFile::read(&path)
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
    let size: usize = env::args()
        .nth(1)
        .map(|x| x.parse::<usize>().unwrap())
        .unwrap_or_else(|| 1024usize);
    let num: usize = env::args()
        .nth(2)
        .map(|x| x.parse::<usize>().unwrap())
        .unwrap_or_else(|| 5usize);
    let output_file: Option<String> = env::args().nth(3);

    println!("Running {} iteration with file size {} bytes", num, size);
    let res = benchmark(num, size, fastread, "fastread").expect("Failed to run benchmark");
    let summary = res.summary();
    println!("Summary: {:#?}", summary);

    if let Some(path) = output_file {
        let mut file = File::create(path)
            .expect("Failed to open output file");
       res.write_as_csv(&mut file) 
            .expect("Failed write output file");
    }
}


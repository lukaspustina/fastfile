use fastfile::prelude::*;
use fastfile_benches::utils::{create_random_test_file};
use std::{
    env,
    io::{self, Write},
    fmt::{self, Display},
    fs::File,
    marker::PhantomData,
    path::{Path},
    time::Instant,
};

trait AsCsv {
    fn write_as_csv<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}

struct Benchmark<'a, T, F: Fn(&T) -> ()> {
    name: &'a str,
    params: &'a[Param<'a, T>],
    functions: Vec<Function<'a, T, F>>,
    iterations: usize,
}

impl<'a, T, F: Fn(&T) -> ()> Benchmark<'a, T, F> {
    fn new(name: &'a str, params: &'a[Param<'a, T>], function_name: &'a str, f: F, iterations: usize) -> Benchmark<'a, T, F> {
        let mut functions = Vec::new();
        let function = Function {
            name: function_name,
            function: f,
            _param_type: PhantomData,
        };
        functions.push(function);
        Benchmark {
            name,
            params,
            functions,
            iterations,
        }
    }

    fn benchmark(&self) -> BenchmarkResult {
        let num_of_samples = self.params.len() * self.functions.len();
        let mut res = BenchmarkResult::new(num_of_samples);

        for f in &self.functions {
            let func = &f.function;
            for p in self.params {
                let mut run_res = BenchmarkResult::new(self.params.len());
                for _ in 0..self.iterations {
                    // TODO: pre_run
                    // fastfile_benches::io::purge_cache(&test_file)?;

                    // measure
                    let time_ns = measure_ns(
                        || func(&p.value)
                    );
                    run_res.add(Sample::new(f.name, p.name, time_ns));
                }
                println!("{}/{}: {}", f.name, p.name, run_res.samples.summary());
                res.samples.append(&mut run_res.samples);
            }
        }

        res
    }
}

fn measure_ns<O, F: Fn() -> O>(func: F) -> u128 {
    let start = Instant::now();
    func();
    start.elapsed().as_nanos()
}

struct Param<'a, T> {
    name: &'a str,
    value: T,
}

impl<'a, T> Param<'a, T> {
    fn new(name: &'a str, value: T) -> Param<'a, T> {
        Param {
            name,
            value,
        }
    }
}

struct Function<'a, T, F> where F: Fn(&T) -> () {
    name: &'a str,
    function: F,
    _param_type: PhantomData<T>,
}


#[derive(Debug)]
struct BenchmarkResult<'a> {
    pub samples: Vec<Sample<'a>>,
}

impl<'a> BenchmarkResult<'a> {
    fn new(num: usize) -> BenchmarkResult<'a> {
        let samples = Vec::with_capacity(num);
        BenchmarkResult {
            samples
        }
    }

    fn add(&mut self, sample: Sample<'a>) {
        self.samples.push(sample)
    }


}

impl<'a> AsCsv for BenchmarkResult<'a> {
    fn write_as_csv<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "strategy,file_size,time")?;
        for s in &self.samples {
            writeln!(writer, "{},size-{},{}", s.name, s.param, s.time_ns)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Sample<'a> {
    pub name: &'a str,
    pub param: &'a str,
    pub time_ns: u128,
}

impl<'a> Sample<'a> {
    fn new(name: &'a str, param: &'a str, time_ns: u128) -> Sample<'a> {
        Sample { name, param, time_ns }
    }
}

trait Summarize {
    fn summary(&self) -> Summary;
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
            min, max, mean
        }
    }
}

impl Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} {} {}]", self.min, self.mean, self.max)
    }
}

impl Summarize for Vec<Sample<'_>> {
    fn summary(&self) -> Summary {
        let mut min = u128::max_value();
        let mut max = u128::min_value();
        let mut sum = 0;

        for s in self.iter() {
            sum += s.time_ns;
            min = min.min(s.time_ns);
            max = max.max(s.time_ns);
        }
        let mean = sum / self.len() as u128;

        Summary::new(min, max, mean)
    }
}

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


    let params = vec![
        Param::new("1024", create_random_test_file(1024).unwrap()),
        Param::new("2048", create_random_test_file(2048).unwrap()),
        Param::new("4096", create_random_test_file(4096).unwrap()),
        Param::new("10240", create_random_test_file(10240).unwrap()),
        Param::new("102400", create_random_test_file(102400).unwrap()),
        Param::new("1024000", create_random_test_file(1024000).unwrap()),
        Param::new("10240000", create_random_test_file(10240000).unwrap()),
    ];

    let benchmark = Benchmark::new(
        "Fastfile read",
        &params,
        "fastread",
        fastread,
        num,
    );

    println!("Running benchmark '{}' with {} params for {} functions", benchmark.name, benchmark.params.len(), benchmark.functions.len());
    let res = benchmark.benchmark();
    println!("Received {} samples (expected {})", res.samples.len(), benchmark.params.len() * benchmark.functions.len() * benchmark.iterations);

    if let Some(path) = output_file {
        let mut file = File::create(path)
            .expect("Failed to open output file");
       res.write_as_csv(&mut file) 
            .expect("Failed write output file");
    }
}


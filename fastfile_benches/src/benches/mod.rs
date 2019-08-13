use byte_unit::Byte;
use std::{
    io::{self, Write},
    fmt::{self, Display},
    time::Instant,
};

pub trait AsCsv {
    fn write_as_csv<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}

pub struct Benchmark<'a, T> {
    name: &'a str,
    params: &'a[Param<T>],
    iterations: usize,
    setup: Option<Box<dyn Fn(&T) -> ()>>,
    functions: Vec<NamedFunction<'a, T>>,
    teardown: Option<Box<dyn Fn(&T) -> ()>>,
}

impl<'a, T> Benchmark<'a, T> {
    pub fn new(name: &'a str, params: &'a[Param<T>], iterations: usize) -> Benchmark<'a, T> {
          Benchmark {
            name,
            params,
            iterations,
            setup: None,
            functions: Vec::new(),
            teardown: None,
        }
    }

    pub fn with_func<F: Fn(&T) -> () + 'static>(name: &'a str, params: &'a[Param<T>], iterations: usize, function_name: &'a str, func: F) -> Benchmark<'a, T> {
        let mut benchmark = Benchmark::new(name, params, iterations);
        let function = NamedFunction {
            name: function_name,
            function: Box::new(func),
        };
        benchmark.functions.push(function);

        benchmark
    }

    pub fn setup<F: Fn(&T) -> () + 'static>(self, func: F) -> Self {
        let mut benchmark = self;
        benchmark.setup = Some(Box::new(func));
        benchmark
    }

    pub fn add_func<F: Fn(&T) -> () + 'static>(self, function_name: &'a str, func: F) -> Self {
        let mut benchmark = self;
        let function = NamedFunction {
            name: function_name,
            function: Box::new(func),
        };
        benchmark.functions.push(function);
        benchmark
    }

    pub fn teardown<F: Fn(&T) -> () + 'static>(self, func: F) -> Self {
        let mut benchmark = self;
        benchmark.teardown = Some(Box::new(func));
        benchmark
    }

    pub fn benchmark(&self) -> BenchmarkResult {
        let num_of_samples = self.params.len() * self.functions.len();
        let mut res = BenchmarkResult::new(num_of_samples);

        println!("Running benchmark '{}' with {} param(s) for {} function(s) and {} iteration(s)\n", self.name, self.params.len(), self.functions.len(), self.iterations);
        for f in &self.functions {
            let func = &f.function;
            for p in self.params {
                let mut run_res = BenchmarkResult::new(self.params.len());
                println!("\t{} / {}", f.name, p.display_name);
                for _ in 0..self.iterations {
                    if let Some(ref setup) = self.setup {
                        setup(&p.value)
                    }

                    let time_ns = measure_ns(
                        || func(&p.value)
                    );
                    run_res.add(Sample::new(f.name, &p.name, time_ns));

                    if let Some(ref teardown) = self.teardown {
                        teardown(&p.value)
                    }
                }
                let summary = run_res.samples.summary();
                println!("\t\t\t{}", summary);
                println!("\t\t\t{}", ThroughputSummary::new(&summary, p.amount));
                res.samples.append(&mut run_res.samples);
            }
        }
        println!("\nReceived {} sample(s) (expected {})", res.samples.len(), self.params.len() * self.functions.len() * self.iterations);

        res
    }
}

fn measure_ns<O, F: Fn() -> O>(func: F) -> u128 {
    let start = Instant::now();
    func();
    start.elapsed().as_nanos()
}

pub struct Param<T> {
    name: String,
    /// value to use for displaying this parameter
    display_name: String,
    amount: usize, 
    value: T,
}

impl<T> Param<T> {
    pub fn new<S: Into<String>>(name: S, display_name: S, amount: usize, value: T) -> Param<T> {
        Param {
            name: name.into(),
            display_name: display_name.into(),
            amount,
            value,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn amount(&self) -> usize {
        self.amount
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

pub struct NamedFunction<'a, T> {
    name: &'a str,
    function: Box<dyn Fn(&T) -> ()>,
}


#[derive(Debug)]
pub struct BenchmarkResult<'a> {
    pub samples: Vec<Sample<'a>>,
}

impl<'a> BenchmarkResult<'a> {
    pub fn new(num: usize) -> BenchmarkResult<'a> {
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
        writeln!(writer, "method,file_size,time")?;
        for s in &self.samples {
            writeln!(writer, "{},{},{}", s.name, s.param, s.time_ns)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Sample<'a> {
    pub name: &'a str,
    pub param: &'a str,
    pub time_ns: u128,
}

impl<'a> Sample<'a> {
    pub fn new(name: &'a str, param: &'a str, time_ns: u128) -> Sample<'a> {
        Sample { name, param, time_ns }
    }
}

pub trait Summarize {
    fn summary(&self) -> Summary;
}

#[derive(Debug)]
pub struct Summary {
    pub min: u128,
    pub max: u128,
    pub mean: u128,
}

impl Summary {
    pub fn new(min: u128, max: u128, mean: u128) -> Summary {
        Summary {
            min, max, mean
        }
    }
}

impl Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let min = self.min as f64 / 1_000_000 as f64;
        let mean = self.mean as f64 / 1_000_000 as f64;
        let max = self.max as f64 / 1_000_000 as f64;
        write!(f, "[{:7.2} ms, {:7.2} ms, {:7.2} ms]", min, mean, max)
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

#[derive(Debug)]
pub struct ThroughputSummary<'a> {
    summary: &'a Summary,
    amount: usize,
}

impl<'a> ThroughputSummary<'a> {
    pub fn new(summary: &'a Summary, amount: usize) -> ThroughputSummary {
        ThroughputSummary {
            summary,
            amount,
        }
    }
}

impl<'a> Display for ThroughputSummary<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let min = self.amount as f64 / (self.summary.min as f64 / 1_000_000_000f64); // run time in secounds
        let mean = self.amount as f64 / (self.summary.mean as f64 / 1_000_000_000f64);
        let max = self.amount as f64 / (self.summary.max as f64 / 1_000_000_000f64); 

        let bytes_min = Byte::from_bytes(min as u128);
        let bytes_mean = Byte::from_bytes(mean as u128);
        let bytes_max = Byte::from_bytes(max as u128);
        write!(f, "[{}/s, {}/s, {}/s]",
            bytes_min.get_appropriate_unit(true).format(2),
            bytes_mean.get_appropriate_unit(true).format(2),
            bytes_max.get_appropriate_unit(true).format(2)
        )
    }
}


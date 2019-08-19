use byte_unit::Byte;
use std::{
    fmt::{self, Display},
    fs::{self, File},
    io::{self, Write},
    path::Path,
    time::Instant,
};

pub trait WriteAsCSV {
    fn write_as_csv<W: Write>(&self, writer: &mut W) -> io::Result<()>;
    fn write_hdr_as_csv<W: Write>(writer: &mut W) -> io::Result<()>;
}

impl WriteAsCSV for () {
    fn write_as_csv<W: Write>(&self, _: &mut W) -> io::Result<()> { Ok(()) }
    fn write_hdr_as_csv<W: Write>(_: &mut W) -> io::Result<()> { Ok(()) }
}

pub struct Benchmark<'a, T, O: WriteAsCSV> {
    name:       &'a str,
    params:     &'a [Param<T>],
    iterations: usize,
    setup:      Option<Box<dyn Fn(&T) -> ()>>,
    functions:  Vec<NamedFunction<'a, T, O>>,
    teardown:   Option<Box<dyn Fn(&T) -> ()>>,
}

impl<'a, T, O: WriteAsCSV> Benchmark<'a, T, O> {
    pub fn new(name: &'a str, params: &'a [Param<T>], iterations: usize) -> Benchmark<'a, T, O> {
        Benchmark {
            name,
            params,
            iterations,
            setup: None,
            functions: Vec::new(),
            teardown: None,
        }
    }

    pub fn with_func<F: Fn(&T) -> O + 'static>(
        name: &'a str,
        params: &'a [Param<T>],
        iterations: usize,
        function_name: &'a str,
        func: F,
    ) -> Benchmark<'a, T, O> {
        let mut benchmark = Benchmark::new(name, params, iterations);
        let function = NamedFunction {
            name:     function_name,
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

    pub fn add_func<F: Fn(&T) -> O + 'static>(self, function_name: &'a str, func: F) -> Self {
        let mut benchmark = self;
        let function = NamedFunction {
            name:     function_name,
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

    pub fn benchmark(&self) -> BenchmarkResult<'a, O> {
        let num_of_samples = self.params.len() * self.functions.len();
        let mut res = BenchmarkResult::new(self.name, num_of_samples);

        println!(
            "Running benchmark '{}' with {} param(s) for {} function(s) and {} iteration(s)\n",
            self.name,
            self.params.len(),
            self.functions.len(),
            self.iterations
        );
        for f in &self.functions {
            let func = &f.function;
            for p in self.params {
                let mut run_res = BenchmarkResult::new(self.name, self.params.len());
                println!("\t{} / {}", f.name, p.display_name);
                for _ in 0..self.iterations {
                    if let Some(ref setup) = self.setup {
                        setup(&p.value)
                    }

                    let (time_ns, f_res) = measure_ns(|| func(&p.value));
                    run_res.add(Sample::new(f.name, &p.name, time_ns, f_res));

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
        println!(
            "\nReceived {} sample(s) (expected {})",
            res.samples.len(),
            self.params.len() * self.functions.len() * self.iterations
        );

        res
    }

    pub fn name(&self) -> &str { self.name }

    pub fn params(&self) -> &[Param<T>] { self.params }
}

fn measure_ns<O, F: Fn() -> O>(func: F) -> (u128, O) {
    let start = Instant::now();
    let res = func();
    let elapsed = start.elapsed().as_nanos();

    (elapsed, res)
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

    pub fn name(&self) -> &str { &self.name }

    pub fn amount(&self) -> usize { self.amount }

    pub fn value(&self) -> &T { &self.value }
}

pub struct NamedFunction<'a, T, O: WriteAsCSV> {
    name:     &'a str,
    function: Box<dyn Fn(&T) -> O>,
}

#[derive(Debug)]
pub struct BenchmarkResult<'a, O: WriteAsCSV> {
    pub benchmark_name: &'a str,
    pub samples:        Vec<Sample<'a, O>>,
}

impl<'a, O: WriteAsCSV> BenchmarkResult<'a, O> {
    pub fn new(benchmark_name: &'a str, num: usize) -> BenchmarkResult<'a, O> {
        let samples = Vec::with_capacity(num);
        BenchmarkResult {
            benchmark_name,
            samples,
        }
    }

    pub fn write_results<P: AsRef<Path>>(&self, results_dir: P) -> io::Result<()> {
        fs::create_dir_all(&results_dir)?;
        let mut output_path = results_dir.as_ref().join(self.benchmark_name);
        output_path.set_extension("csv");
        println!("Writing benchmark results to \"{}\"", &output_path.to_string_lossy());
        let mut file = File::create(output_path)?;
        self.write_as_csv(&mut file)
    }

    fn add(&mut self, sample: Sample<'a, O>) { self.samples.push(sample) }

    fn write_as_csv<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write!(writer, "method,file_size,time,")?;
        O::write_hdr_as_csv(writer)?;
        writeln!(writer, "")?;
        for s in &self.samples {
            write!(writer, "{},{},{},", s.name, s.param, s.time_ns)?;
            s.extra.write_as_csv(writer)?;
            writeln!(writer, "")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Sample<'a, O: WriteAsCSV> {
    pub name:    &'a str,
    pub param:   &'a str,
    pub time_ns: u128,
    pub extra: O,
}

impl<'a, O: WriteAsCSV> Sample<'a, O> {
    pub fn new(name: &'a str, param: &'a str, time_ns: u128, extra: O) -> Sample<'a, O> { Sample { name, param, time_ns, extra } }
}

pub trait Summarize {
    fn summary(&self) -> Summary;
}

#[derive(Debug)]
pub struct Summary {
    pub min:  u128,
    pub max:  u128,
    pub mean: u128,
}

impl Summary {
    pub fn new(min: u128, max: u128, mean: u128) -> Summary { Summary { min, max, mean } }
}

impl Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let min = self.min as f64 / 1_000_000f64;
        let mean = self.mean as f64 / 1_000_000f64;
        let max = self.max as f64 / 1_000_000f64;
        write!(f, "[{:7.2} ms, {:7.2} ms, {:7.2} ms]", min, mean, max)
    }
}

impl<O: WriteAsCSV> Summarize for Vec<Sample<'_, O>> {
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
    amount:  usize,
}

impl<'a> ThroughputSummary<'a> {
    pub fn new(summary: &'a Summary, amount: usize) -> ThroughputSummary { ThroughputSummary { summary, amount } }
}

impl<'a> Display for ThroughputSummary<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let min = self.amount as f64 / (self.summary.min as f64 / 1_000_000_000f64); // run time in secounds
        let mean = self.amount as f64 / (self.summary.mean as f64 / 1_000_000_000f64);
        let max = self.amount as f64 / (self.summary.max as f64 / 1_000_000_000f64);

        let bytes_min = Byte::from_bytes(min as u128);
        let bytes_mean = Byte::from_bytes(mean as u128);
        let bytes_max = Byte::from_bytes(max as u128);
        write!(
            f,
            "[{}/s, {}/s, {}/s]",
            bytes_min.get_appropriate_unit(true).format(2),
            bytes_mean.get_appropriate_unit(true).format(2),
            bytes_max.get_appropriate_unit(true).format(2)
        )
    }
}

use csv::WriterBuilder;
use egg::*;
use regex;
use std::fs;
use std::fs::*;
use std::io::Read;
use std::time;
use structopt::StructOpt;

pub fn default_runner<L: Language, N: Analysis<L> + Default>() -> Runner<L, N> {
    // let opt = Opt::from_args();
    let mut runner = Runner::new(N::default())
        .with_node_limit(usize::MAX)
        .with_iter_limit(usize::MAX)
        .with_time_limit(time::Duration::from_secs(u64::MAX));
    runner = runner.with_scheduler(egg::SimpleScheduler);
    runner
}

#[derive(Debug, StructOpt)]
#[structopt(name = "egg-smol-benchmarks")]
pub(crate) struct Opt {
    #[structopt(long)]
    disable_egg: bool,
    #[structopt(long)]
    disable_egglog: bool,
    // repeat should be an odd number
    #[structopt(long, default_value = "3")]
    repeat: usize,
    #[structopt(long, default_value = "200")]
    iter_size: usize,
}

pub fn get_text(name: &str) -> Option<String> {
    let mut text = String::default();
    File::open(format!("src/egglog/{}.egg", name))
        .ok()?
        .read_to_string(&mut text)
        .ok()?;
    Some(text)
}

trait Benchmark {
    fn name(&self) -> String;
    fn run_egg(&self) -> usize;
    fn egglog_text(&self) -> Option<String>;
    fn run_egglog_with_engine(&self, mut egraph: egg_smol::EGraph) -> usize {
        let msgs = egraph
            .parse_and_run_program(&self.egglog_text().unwrap())
            .unwrap();
        log::info!("===== egglog =====");
        let mut report = None;
        let mut db_size = 0;
        let re = regex::Regex::new("has size ([0-9]+)").unwrap();
        for msg in msgs.iter().rev() {
            if msg.starts_with("Ran ") {
                // we only take the last report
                if report.is_none() {
                    report = Some(msg);
                }
            }
            if msg.starts_with("Function ") {
                let cap = re.captures(msg).unwrap();
                db_size += cap[1].parse::<usize>().unwrap();
            }
        }
        if let Some(report) = report {
            log::info!("{}", report);
        } else {
            log::info!("No egglog performance report for {}", self.name());
        }
        db_size
    }
    fn run_egglog(&mut self) -> usize;

    fn run_egglognaive(&mut self) -> usize;
}

#[derive(Clone, Debug, Serialize)]
enum Engine {
    Egg,
    Egglog,
    EgglogNaive,
}
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
struct BenchmarkRecord {
    benchmark: String,
    engine: Engine,
    time: String,
    size: usize,
}

mod math;
#[derive(Default)]
struct BenchmarkRunner;

impl BenchmarkRunner {
    pub fn run(&self, benches: Vec<Box<dyn Benchmark>>) -> Vec<BenchmarkRecord> {
        let mut records = vec![];
        for (i, mut bench) in benches.into_iter().enumerate() {
            let r = self.run_one(&mut bench);
            records.extend(r.into_iter());

            if i % 5 == 0 {
                let mut wtr = WriterBuilder::new().has_headers(false).from_writer(vec![]);
                for record in records.iter() {
                    wtr.serialize(record).unwrap();
                }
                let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
                fs::write("benchmarks.csv", data).unwrap();
            }
        }
        records
    }

    pub fn run_one(&self, bench: &mut Box<dyn Benchmark>) -> Vec<BenchmarkRecord> {
        let opt = Opt::from_args();
        let mut egg_duration = None;
        let mut egglog_duration = None;
        let mut egglognaive_duration = None;
        let mut records = vec![];
        if !opt.disable_egg {
            let mut durations = vec![];
            let mut size = 0;
            for _ in 0..opt.repeat {
                let egg_start_time = time::Instant::now();
                size = bench.run_egg();
                durations.push(time::Instant::now() - egg_start_time);
            }
            durations.sort();
            egg_duration = Some(durations[opt.repeat / 2]);
            records.push(BenchmarkRecord {
                size,
                benchmark: bench.name().to_string(),
                engine: Engine::Egg,
                time: egg_duration.unwrap().as_nanos().to_string(),
            });
        }

        if !opt.disable_egglog {
            let mut durations = vec![];
            let mut size = 0;
            for _ in 0..opt.repeat {
                let egglog_start_time = time::Instant::now();
                size = bench.run_egglog();
                durations.push(time::Instant::now() - egglog_start_time);
            }
            durations.sort();
            egglog_duration = Some(durations[opt.repeat / 2]);
            records.push(BenchmarkRecord {
                size,
                benchmark: bench.name().to_string(),
                engine: Engine::Egglog,
                time: egglog_duration.unwrap().as_nanos().to_string(),
            });
        }

        if !opt.disable_egglog {
            let mut durations = vec![];
            let mut size = 0;
            for _ in 0..opt.repeat {
                let egglognaive_start_time = time::Instant::now();
                size = bench.run_egglognaive();
                durations.push(time::Instant::now() - egglognaive_start_time);
            }
            durations.sort();
            egglognaive_duration = Some(durations[opt.repeat / 2]);
            records.push(BenchmarkRecord {
                size,
                benchmark: bench.name().to_string(),
                engine: Engine::EgglogNaive,
                time: egglognaive_duration.unwrap().as_nanos().to_string(),
            });
        }

        if !opt.disable_egg && !opt.disable_egglog {
            eprintln!(
                "On benchmark {:?}, egglog spent {:.3}s, egglog-naive spent {:.3}s and egg spent {:.3}s, egglog/egg: {:?}",
                bench.name(),
                egglog_duration.unwrap().as_secs_f64(),
                egglognaive_duration.unwrap().as_secs_f64(),
                egg_duration.unwrap().as_secs_f64(),
                egglog_duration.unwrap().as_secs_f64() / egg_duration.unwrap().as_secs_f64()
            );
        }

        if opt.disable_egg && !opt.disable_egglog {
            eprintln!(
                "On benchmark {:?}, egglog spent {:.3}s and egglog-naive spent {:.3}s",
                bench.name(),
                egglog_duration.unwrap().as_secs_f64(),
                egglognaive_duration.unwrap().as_secs_f64()
            );
        }

        records
    }
}

fn main() {
    let opt = Opt::from_args();
    env_logger::init();
    let mut benches = vec![];
    for i in 1..opt.iter_size + 1 {
        // for i in opt.iter_size..opt.iter_size + 1 {
        benches.push(math::run_n::new(i));
    }
    let records = BenchmarkRunner::default().run(benches);
    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(vec![]);
    for record in records {
        wtr.serialize(record).unwrap();
    }
    let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
    fs::write("benchmarks.csv", data).unwrap();
}

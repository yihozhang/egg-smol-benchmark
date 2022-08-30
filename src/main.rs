use std::fs::*;
use std::io::Read;
use std::time;

trait Bench {
    fn name(&self) -> &str;
    fn run_egg(&self);
    fn egglog_text(&self) -> Option<String> {
        let mut text = String::default();
        File::open(format!("benchmarks/{}.egg", self.name()))
            .ok()?
            .read_to_string(&mut text)
            .ok()?;
        Some(text)
    }
    fn run_egglog(&self) {
        let mut egraph = egg_smol::EGraph::default();
        egraph
            .parse_and_run_program(&self.egglog_text().unwrap())
            .unwrap();
    }
}

#[derive(Clone, Debug)]
enum Engine {
    Egg,
    Egglog,
}

#[derive(Clone, Debug)]
struct BenchRecord {
    benchmark: String,
    engine: Engine,
    time: String,
}

mod math;

#[derive(Default)]
struct BenchRunner;

impl BenchRunner {
    pub fn run(&self, benches: &Vec<Box<dyn Bench>>) {
        let mut records = vec![];
        for bench in benches {
            let r = self.run_one(bench);
            records.extend(r.into_iter());
        }
    }

    pub fn run_one(&self, bench: &Box<dyn Bench>) -> Vec<BenchRecord> {
        let egg_start_time = time::Instant::now();
        bench.run_egg();
        let egg_duration = time::Instant::now() - egg_start_time;
        let record1 = BenchRecord {
            benchmark: bench.name().to_string(),
            engine: Engine::Egg,
            time: egg_duration.as_nanos().to_string(),
        };

        let egglog_start_time = time::Instant::now();
        bench.run_egglog();
        let egglog_duration = time::Instant::now() - egglog_start_time;
        let record2 = BenchRecord {
            benchmark: bench.name().to_string(),
            engine: Engine::Egglog,
            time: egg_duration.as_nanos().to_string(),
        };

        println!(
            "On benchmark {:?}, egglog spent {:.3}s and egg spent {:.3}s, egglog/egg: {:?}",
            bench.name(),
            egglog_duration.as_secs_f64(),
            egg_duration.as_secs_f64(),
            egglog_duration.as_secs_f64() / egg_duration.as_secs_f64()
        );

        vec![record1, record2]
    }
}
fn benches() -> Vec<Box<dyn Bench>> {
    vec![Box::new(math::ac::new())]
}
fn main() {
    BenchRunner::default().run(&benches());
}

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
        egraph.parse_and_run_program(&self.egglog_text().unwrap()).unwrap();
    }
}

struct BenchRecord {
    benchmark: String,
    node_size: usize,
    class_size: usize,
    algo: String,
    pattern: String,
    time: String,
    result_size: usize,
    repeat_time: usize,
}

mod math;

#[derive(Default)]
struct BenchRunner;

impl BenchRunner {
    pub fn run(&self, benches: &Vec<Box<dyn Bench>>) {
        for bench in benches {
            let (d1, d2) = self.run_one(bench);
            println!(
                "On benchmark {:?}, egglog spent {:.3} and egg spent {:.3}, egglog/egg: {:?}",
                bench.name(),
                d2.as_secs_f64(),
                d1.as_secs_f64(),
                d2.as_secs_f64() / d1.as_secs_f64()
            )
        }
    }

    pub fn run_one(&self, bench: &Box<dyn Bench>) -> (time::Duration, time::Duration) {
        let egg_start_time = time::Instant::now();
        bench.run_egg();
        let egg_duration = time::Instant::now() - egg_start_time;

        let egglog_start_time = time::Instant::now();
        bench.run_egglog();
        let egglog_duration = time::Instant::now() - egglog_start_time;
        (egg_duration, egglog_duration)
    }
}
fn benches() -> Vec<Box<dyn Bench>> {
    vec![
        Box::new(math::ac::new()),
    ]
}
fn main() {
    BenchRunner::default().run(&benches());
}

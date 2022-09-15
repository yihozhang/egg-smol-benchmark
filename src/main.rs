use std::fs::*;
use std::io::Read;
use std::time;

pub fn get_text(name: &str) -> Option<String> {
    let mut text = String::default();
    File::open(format!("src/egglog/{}.egg", name))
        .ok()?
        .read_to_string(&mut text)
        .ok()?;
    Some(text)
}

trait Bench {
    fn name(&self) -> &str;
    fn run_egg(&self);
    fn egglog_text(&self) -> Option<String> {
        get_text(self.name())
    }
    fn run_egglog(&self) {
        let mut egraph = egg_smol::EGraph::default();
        let msgs = egraph
            .parse_and_run_program(&self.egglog_text().unwrap())
            .unwrap();
        log::info!("===== egglog =====");
        let mut report = None;
        for msg in msgs.iter().rev() {
            if msg.starts_with("Ran ") {
                if report.is_none() {
                    report = Some(msg);
                    break;
                } else{
                    log::error!("multiple egglog performance report for {}", self.name());
                }
            }
        }
        if let Some(report) = report {
            log::info!("{}", report);
        } else {
            log::info!("No egglog performance report for {}", self.name());
        }
    }
}

#[derive(Clone, Debug, Serialize)]
enum Engine {
    Egg,
    Egglog,
}
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
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
    vec![
        Box::new(math::ac::new()),
        Box::new(math::simplify_root::new()),
        Box::new(math::simplify_factor::new())
    ]
}
fn main() {
    env_logger::init();
    BenchRunner::default().run(&benches());
}

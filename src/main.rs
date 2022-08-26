use std::time;

trait Bench {
    fn name(&self) -> String;
    fn run_egg(&self);
    fn run_egglog(&self);
}

mod math;

#[derive(Default)]
struct BenchRunner;

impl BenchRunner {
    pub fn run(&self, benches: &Vec<Box<dyn Bench>>) {
        for bench in benches {
            let (d1, d2) = self.run_one(bench);
            println!(
                "On benchmark {:?}, egglog spent {:?} and egg spent {:?}, egglog/egg: {:?}",
                bench.name(),
                d2.as_micros(),
                d1.as_micros(),
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
    vec![Box::new(math::ac::new())]
}
fn main() {
    BenchRunner::default().run(&benches());
}

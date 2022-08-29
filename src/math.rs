use crate::Bench;
use egg::rewrite as rw;
use egg::*;
use num_rational::BigRational;
use std::default::*;
use std::*;

pub type Constant = BigRational;
pub mod ac {
    use super::*;

    define_language! {
        pub enum Math {
            "+" = Add([Id; 2]),
            Constant(Constant),
        }
    }
    // type EGraph = egg::EGraph<Math, ()>;
    type Rewrite = egg::Rewrite<Math, ()>;

    #[derive(Default)]
    pub struct AC {
        name: String,
        num_iter: usize,
    }

    pub fn new() -> AC {
        AC {
            name: "math_ac_10".into(),
            num_iter: 10,
        }
    }

    impl AC {
        fn rewrites(&self) -> Vec<Rewrite> {
            vec![
                rw!("comm-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
                rw!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
            ]
        }
    }

    impl Bench for AC {
        fn name(&self) -> &str {
            &self.name
        }

        fn run_egg(&self) {
            let start_expr = "(+ 1 (+ 2 (+ 3 (+ 4 (+ 5 (+ 6 (+ 7 (+ 8 (+ 9 10)))))))))"
                .parse()
                .unwrap();
            let end_expr = "(+ 10 (+ 9 (+ 8 (+ 7 (+ 6 (+ 5 (+ 4 (+ 3 (+ 2 1)))))))))"
                .parse()
                .unwrap();
            let runner = Runner::default()
                .with_iter_limit(self.num_iter)
                .with_scheduler(SimpleScheduler)
                .with_expr(&start_expr)
                .run(&self.rewrites());
            let egraph = &runner.egraph;
            assert!(egraph.equivs(&start_expr, &end_expr).len() == 1);
        }
    }
}

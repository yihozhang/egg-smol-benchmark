use crate::Benchmark;
use crate::*;
use std::default::*;
use std::*;

pub mod run_n {
    use super::*;

    pub(crate) fn new(n: usize) -> Box<dyn Benchmark> {
        Box::new(RunN {
            seminaive: false,
            n,
        })
    }
    pub struct RunN {
        seminaive: bool,
        n: usize,
    }

    impl Benchmark for RunN {
        fn name(&self) -> String {
            format!("math-run-{}", self.n)
        }

        fn run_egg(&self) -> usize {
            return 0;
        }

        fn egglog_text(&self) -> Option<String> {
            let mut src = crate::get_text(&"math_full")?;
            src.push_str(
                &format!(
                r#"
            (Integral (Ln (Var "x")) (Var "x"))
            (Integral (Add (Var "x") (Cos (Var "x"))) (Var "x"))
            (Integral (Mul (Cos (Var "x")) (Var "x")) (Var "x"))
            (Diff (Var "x") (Add (Const (rational 1 1)) (Mul (Const (rational 2 1)) (Var "x"))))
            (Diff (Var "x") (Sub (Pow (Var "x") (Const (rational 3 1))) (Mul (Const (rational 7 1)) (Pow (Var "x") (Const (rational 2 1))))))
            (Add (Mul (Var "y") (Add (Var "x") (Var "y"))) (Sub (Add (Var "x") (Const (rational 2 1))) (Add (Var "x") (Var "x"))))
            (Div (Const (rational 1 1))
                                    (Sub (Div (Add (Const (rational 1 1))
                                                (Sqrt (Var "five")))
                                            (Const (rational 2 1)))
                                        (Div (Sub (Const (rational 1 1))
                                                (Sqrt (Var "five")))
                                            (Const (rational 2 1)))))
            (run :naive :match-limit 1000 {})
            (run :{} :match-limit 1000000000 1)
            (print-size Diff)
            (print-size Integral)
            (print-size Add)
            (print-size Sub)
            (print-size Mul)
            (print-size Div)
            (print-size Pow)
            (print-size Ln)
            (print-size Sqrt)
            (print-size Sin)
            (print-size Cos)
            (print-size Const)
            (print-size Var)
            "#,
            self.n-1,
            if self.seminaive { "seminaive" } else { "naive" }
            ));
            Some(src)
        }

        fn run_egglog(&mut self) -> usize {
            let mut egraph = egg_smol::EGraph::default();
            // we grow the e-graph with naive execution.
            egraph.seminaive = false;
            self.seminaive = true;
            self.run_egglog_with_engine(egraph)
        }
        fn run_egglognaive(&mut self) -> usize {
            let mut egraph = egg_smol::EGraph::default();
            egraph.seminaive = false;
            self.seminaive = false;
            self.run_egglog_with_engine(egraph)
        }
    }
}

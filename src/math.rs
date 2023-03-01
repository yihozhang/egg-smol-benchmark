use crate::Benchmark;
use crate::*;
use std::default::*;
use std::*;

pub mod math_egg_src {
    use egg::{rewrite as rw, *};
    use num_rational::Rational64;
    pub type Rewrite = egg::Rewrite<Math, ()>;

    pub type Constant = Rational64;

    define_language! {
        pub enum Math {
            "d" = Diff([Id; 2]),
            "i" = Integral([Id; 2]),

            "+" = Add([Id; 2]),
            "-" = Sub([Id; 2]),
            "*" = Mul([Id; 2]),
            "/" = Div([Id; 2]),
            "pow" = Pow([Id; 2]),
            "ln" = Ln(Id),
            "sqrt" = Sqrt(Id),

            "sin" = Sin(Id),
            "cos" = Cos(Id),

            Constant(Constant),
            Symbol(Symbol),
        }
    }

    pub fn rules() -> Vec<Rewrite> {
        vec![
            rw!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
            rw!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
            rw!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
            rw!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),
            rw!("sub-canon"; "(- ?a ?b)" => "(+ ?a (* -1 ?b))"),
            rw!("zero-add"; "(+ ?a 0)" => "?a"),
            rw!("zero-mul"; "(* ?a 0)" => "0"),
            rw!("one-mul";  "(* ?a 1)" => "?a"),
            rw!("cancel-sub"; "(- ?a ?a)" => "0"),
            rw!("distribute"; "(* ?a (+ ?b ?c))"        => "(+ (* ?a ?b) (* ?a ?c))"),
            rw!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),
            rw!("pow-mul"; "(* (pow ?a ?b) (pow ?a ?c))" => "(pow ?a (+ ?b ?c))"),
            rw!("pow1"; "(pow ?x 1)" => "?x"),
            rw!("pow2"; "(pow ?x 2)" => "(* ?x ?x)"),
            rw!("d-add"; "(d ?x (+ ?a ?b))" => "(+ (d ?x ?a) (d ?x ?b))"),
            rw!("d-mul"; "(d ?x (* ?a ?b))" => "(+ (* ?a (d ?x ?b)) (* ?b (d ?x ?a)))"),
            rw!("d-sin"; "(d ?x (sin ?x))" => "(cos ?x)"),
            rw!("d-cos"; "(d ?x (cos ?x))" => "(* -1 (sin ?x))"),
            rw!("i-one"; "(i 1 ?x)" => "?x"),
            rw!("i-cos"; "(i (cos ?x) ?x)" => "(sin ?x)"),
            rw!("i-sin"; "(i (sin ?x) ?x)" => "(* -1 (cos ?x))"),
            rw!("i-sum"; "(i (+ ?f ?g) ?x)" => "(+ (i ?f ?x) (i ?g ?x))"),
            rw!("i-dif"; "(i (- ?f ?g) ?x)" => "(- (i ?f ?x) (i ?g ?x))"),
            rw!("i-parts"; "(i (* ?a ?b) ?x)" =>
                "(- (* ?a (i ?b ?x)) (i (* (d ?x ?a) (i ?b ?x)) ?x))"),
        ]
    }
}

pub mod run_n {
    use super::math_egg_src::*;
    use super::*;

    pub(crate) fn new(n: usize) -> Box<dyn Benchmark> {
        Box::new(RunN { n })
    }
    pub struct RunN {
        n: usize,
    }

    impl Benchmark for RunN {
        fn name(&self) -> String {
            format!("math-run-{}", self.n)
        }

        fn run_egg(&self) -> usize {
            let start_exprs = vec![
                "(i (ln x) x)",
                "(i (+ x (cos x)) x)",
                "(i (* (cos x) x) x)",
                "(d x (+ 1 (* 2 x)))",
                "(d x (- (pow x 3) (* 7 (pow x 2))))",
                "(+ (* y (+ x y)) (- (+ x 2) (+ x x)))",
                "(/ 1 (- (/ (+ 1 (sqrt five)) 2) (/ (- 1 (sqrt five)) 2)))",
            ];
            let mut runner = default_runner();
            runner = runner
                .with_scheduler(egg::BackoffScheduler::default())
                // .with_scheduler(egg::SimpleScheduler)
                .with_iter_limit(self.n);
            for start_expr in start_exprs.iter() {
                runner = runner.with_expr(&start_expr.parse().unwrap());
            }
            runner = runner.run(&rules());
            assert!(matches!(
                runner.stop_reason,
                Some(StopReason::IterationLimit(_))
            ));

            let report = runner.report();
            log::info!("===== egg =====");
            log::info!("{}", report);
            report.egraph_nodes
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
            (run {})
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
            self.n
            ));
            Some(src)
        }

        fn run_egglog(&mut self) -> usize {
            let mut egraph = egg_smol::EGraph::default();
            egraph.match_limit = 1000;
            // egraph.match_limit = usize::MAX;
            self.run_egglog_with_engine(egraph)
        }
        fn run_egglognaive(&mut self) -> usize {
            let mut egraph = egg_smol::EGraph::default();
            egraph.match_limit = 1000;
            // egraph.match_limit = usize::MAX;
            egraph.seminaive = false;
            self.run_egglog_with_engine(egraph)
        }
    }
}

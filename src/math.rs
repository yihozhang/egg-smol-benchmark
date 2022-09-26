use crate::Bench;
use crate::*;
use egg::rewrite as rw;
use egg::*;
use std::default::*;
use std::*;

pub fn default_runner<L: Language, N: Analysis<L> + Default>() -> Runner<L, N> {
    let opt = Opt::from_args();
    let mut runner = Runner::new(N::default())
        .with_node_limit(usize::MAX)
        .with_iter_limit(usize::MAX);
    if opt.egg_uses_backoff_scheduler {
        runner = runner.with_scheduler(egg::BackoffScheduler::default());
    } else {
        runner = runner.with_scheduler(egg::SimpleScheduler);
    }
    runner
}

fn run_and_check<L: Language + FromOp + 'static, N: Analysis<L>>(
    start_expr: &str,
    end_expr: &str,
    mut runner: Runner<L, N>,
    rules: Vec<Rewrite<L, N>>,
    early_check: bool,
) -> Runner<L, N> {
    let start_expr: RecExpr<L> = start_expr.parse().unwrap();
    let s = start_expr.clone();

    let end_expr: RecExpr<L> = end_expr.parse().unwrap();
    let e = end_expr.clone();

    if early_check {
        runner = runner.with_hook(move |r| {
            let egraph = &r.egraph;
            if egraph.lookup_expr(&start_expr) == egraph.lookup_expr(&end_expr) {
                Err("Proved all goals".into())
            } else {
                Ok(())
            }
        });
    }
    let runner = runner.with_expr(&s).run(&rules);

    let report = runner.report();
    log::info!("===== egg =====");
    log::info!("{}", report);

    let egraph = &runner.egraph;
    assert!(egraph.lookup_expr(&s) == egraph.lookup_expr(&e));

    runner
}

pub mod ac {
    use super::*;
    use num_rational::Rational64;
    pub type Constant = Rational64;

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
    }

    pub(crate) fn new() -> impl Bench {
        AC {
            name: "math_ac_10".into(),
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
            let num_iter = 10;
            let runner = default_runner()
                .with_iter_limit(num_iter)
                .with_scheduler(SimpleScheduler)
                .with_node_limit(usize::MAX)
                .with_time_limit(time::Duration::MAX);
            let runner = run_and_check(
                "(+ 1 (+ 2 (+ 3 (+ 4 (+ 5 (+ 6 (+ 7 (+ 8 (+ 9 10)))))))))",
                "(+ 10 (+ 9 (+ 8 (+ 7 (+ 6 (+ 5 (+ 4 (+ 3 (+ 2 1)))))))))",
                runner,
                self.rewrites(),
                false,
            );
            assert_eq!(runner.iterations.len(), num_iter);
        }
    }
}

pub mod math_egg_src {
    use egg::{rewrite as rw, *};
    use num_rational::Rational64;
    use num_traits::{Zero, One};

    pub type EGraph = egg::EGraph<Math, ConstantFold>;
    pub type Rewrite = egg::Rewrite<Math, ConstantFold>;

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

    // You could use egg::AstSize, but this is useful for debugging, since
    // it will really try to get rid of the Diff operator
    pub struct MathCostFn;
    impl egg::CostFunction<Math> for MathCostFn {
        type Cost = usize;
        fn cost<C>(&mut self, enode: &Math, mut costs: C) -> Self::Cost
        where
            C: FnMut(Id) -> Self::Cost,
        {
            let op_cost = match enode {
                Math::Diff(..) => 100,
                Math::Integral(..) => 100,
                _ => 1,
            };
            enode.fold(op_cost, |sum, i| sum + costs(i))
        }
    }

    #[derive(Default)]
    pub struct ConstantFold;
    impl Analysis<Math> for ConstantFold {
        type Data = Option<Constant>;

        fn make(egraph: &EGraph, enode: &Math) -> Self::Data {
            let x = |i: &Id| egraph[*i].data.as_ref().map(|d| d);
            Some(match enode {
                Math::Constant(c) => *c,
                Math::Add([a, b]) => x(a)? + x(b)?,
                Math::Sub([a, b]) => x(a)? - x(b)?,
                Math::Mul([a, b]) => x(a)? * x(b)?,
                Math::Div([a, b]) if x(b) != Some(&Rational64::zero()) => x(a)? / x(b)?,
                _ => return None,
            })
        }

        fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
            merge_option(to, from, |a, b| {
                assert_eq!(a, &b, "Merged non-equal constants");
                DidMerge(false, false)
            })
        }

        fn modify(egraph: &mut EGraph, id: Id) {
            let class = &egraph[id];
            if let Some(c) = class.data {
                let added = egraph.add(Math::Constant(c));
                egraph.union(id, added);
                // to prune, uncomment this out
                // Note: we don't to prune because it's generally bad
                // and even worse in egglog
                // egraph[id].nodes.retain(|n| n.is_leaf());

                #[cfg(debug_assertions)]
                egraph[id].assert_unique_leaves();
            }
        }
    }

    fn is_const_or_distinct_var(v: &str, w: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
        let v = v.parse().unwrap();
        let w = w.parse().unwrap();
        move |egraph, _, subst| {
            egraph.find(subst[v]) != egraph.find(subst[w])
                && (egraph[subst[v]].data.is_some()
                    || egraph[subst[v]]
                        .nodes
                        .iter()
                        .any(|n| matches!(n, Math::Symbol(..))))
        }
    }

    fn is_const(var: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
        let var = var.parse().unwrap();
        move |egraph, _, subst| egraph[subst[var]].data.is_some()
    }

    fn is_sym(var: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
        let var = var.parse().unwrap();
        move |egraph, _, subst| {
            egraph[subst[var]]
                .nodes
                .iter()
                .any(|n| matches!(n, Math::Symbol(..)))
        }
    }

    // NOTE: This is different from the test suite, 
    // because we are doing a sound is_not_zero analysis here
    fn is_not_zero(var: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
        let var = var.parse().unwrap();
        move |egraph, _, subst| {
            if let Some(n) = &egraph[subst[var]].data {
                !n.is_zero()
            } else {
                false
            }
        }
    }

    fn is_not_zero_soft(var: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
        let var = var.parse().unwrap();
        move |egraph, _, subst| {
            if let Some(n) = &egraph[subst[var]].data {
                !n.is_zero()
            } else {
                true
            }
        }
    }

    fn is_not_one_soft(var: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
        let var = var.parse().unwrap();
        move |egraph, _, subst| {
            if let Some(n) = &egraph[subst[var]].data {
                !n.is_one()
            } else {
                true
            }
        }
    }

    pub fn rules() -> Vec<Rewrite> {
        vec![
            rw!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
            rw!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
            rw!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
            rw!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),
            rw!("sub-canon"; "(- ?a ?b)" => "(+ ?a (* -1 ?b))"),
            rw!("div-canon"; "(/ ?a ?b)" => "(* ?a (pow ?b -1))" if is_not_zero("?b")),
            // rw!("canon-sub"; "(+ ?a (* -1 ?b))"   => "(- ?a ?b)"),
            // rw!("canon-div"; "(* ?a (pow ?b -1))" => "(/ ?a ?b)" if is_not_zero("?b")),
            rw!("zero-add"; "(+ ?a 0)" => "?a"),
            rw!("zero-mul"; "(* ?a 0)" => "0"),
            rw!("one-mul";  "(* ?a 1)" => "?a"),
            // The two rules below are different from the egg test suite.
            // This is because the two rules will explode under simple scheduler.
            rw!("add-zero"; "?a" => "(+ ?a 0)" if is_not_zero_soft("?a")),
            rw!("mul-one";  "?a" => "(* ?a 1)" if is_not_one_soft("?a")),
            rw!("cancel-sub"; "(- ?a ?a)" => "0"),
            rw!("cancel-div"; "(/ ?a ?a)" => "1" if is_not_zero("?a")),
            rw!("distribute"; "(* ?a (+ ?b ?c))"        => "(+ (* ?a ?b) (* ?a ?c))"),
            rw!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),
            rw!("pow-mul"; "(* (pow ?a ?b) (pow ?a ?c))" => "(pow ?a (+ ?b ?c))"),
            rw!("pow0"; "(pow ?x 0)" => "1"
        if is_not_zero("?x")),
            rw!("pow1"; "(pow ?x 1)" => "?x"),
            rw!("pow2"; "(pow ?x 2)" => "(* ?x ?x)"),
            rw!("pow-recip"; "(pow ?x -1)" => "(/ 1 ?x)"
        if is_not_zero("?x")),
            rw!("recip-mul-div"; "(* ?x (/ 1 ?x))" => "1" if is_not_zero("?x")),
            rw!("d-variable"; "(d ?x ?x)" => "1" if is_sym("?x")),
            rw!("d-constant"; "(d ?x ?c)" => "0" if is_sym("?x") if is_const_or_distinct_var("?c", "?x")),
            rw!("d-add"; "(d ?x (+ ?a ?b))" => "(+ (d ?x ?a) (d ?x ?b))"),
            rw!("d-mul"; "(d ?x (* ?a ?b))" => "(+ (* ?a (d ?x ?b)) (* ?b (d ?x ?a)))"),
            rw!("d-sin"; "(d ?x (sin ?x))" => "(cos ?x)"),
            rw!("d-cos"; "(d ?x (cos ?x))" => "(* -1 (sin ?x))"),
            rw!("d-ln"; "(d ?x (ln ?x))" => "(/ 1 ?x)" if is_not_zero("?x")),
            rw!("d-power";
                "(d ?x (pow ?f ?g))" =>
                "(* (pow ?f ?g)
                    (+ (* (d ?x ?f)
                        (/ ?g ?f))
                    (* (d ?x ?g)
                        (ln ?f))))"
                if is_not_zero("?f")
                if is_not_zero("?g")
            ),
            rw!("i-one"; "(i 1 ?x)" => "?x"),
            rw!("i-power-const"; "(i (pow ?x ?c) ?x)" =>
                "(/ (pow ?x (+ ?c 1)) (+ ?c 1))" if is_const("?c")),
            rw!("i-cos"; "(i (cos ?x) ?x)" => "(sin ?x)"),
            rw!("i-sin"; "(i (sin ?x) ?x)" => "(* -1 (cos ?x))"),
            rw!("i-sum"; "(i (+ ?f ?g) ?x)" => "(+ (i ?f ?x) (i ?g ?x))"),
            rw!("i-dif"; "(i (- ?f ?g) ?x)" => "(- (i ?f ?x) (i ?g ?x))"),
            rw!("i-parts"; "(i (* ?a ?b) ?x)" =>
                "(- (* ?a (i ?b ?x)) (i (* (d ?x ?a) (i ?b ?x)) ?x))"),
        ]
    }
}

pub mod simplify_root {
    use super::math_egg_src::*;
    use super::*;

    pub(crate) fn new() -> impl Bench {
        SimplifyRoot {}
    }
    pub struct SimplifyRoot {}

    impl Bench for SimplifyRoot {
        fn name(&self) -> &str {
            &"math_simplify_root"
        }

        fn run_egg(&self) {
            let start_expr = &"
                (/ 1
                    (- (/ (+ 1 (sqrt five))
                            2)
                        (/ (- 1 (sqrt five))
                            2)))";
            let end_expr = &"(/ 1 (sqrt five))";
            let _runner = run_and_check(
                start_expr,
                end_expr,
                default_runner(),
                rules(),
                true,
            );
            // assert!(matches!(runner.stop_reason, Some(StopReason::NodeLimit(_))));
        }

        fn egglog_text(&self) -> Option<String> {
            let mut src = crate::get_text(&"math_full")?;
            src.push_str(
                &r#"
            (define start-expr
                (Div (Const (rational 1 1))
                     (Sub (Div (Add (Const (rational 1 1))
                                    (Sqrt (Var "five")))
                               (Const (rational 2 1)))
                          (Div (Sub (Const (rational 1 1))
                                    (Sqrt (Var "five")))
                               (Const (rational 2 1))))))
            (run 11)
            (define end-expr
                (Div (Const (rational 1 1))
                     (Sqrt (Var "five"))))
            (check (= start-expr end-expr))
            "#,
            );
            Some(src)
        }
    }
}

pub mod simplify_factor {
    use super::math_egg_src::*;
    use super::*;

    pub(crate) fn new() -> impl Bench {
        SimplifyFactor {}
    }
    pub struct SimplifyFactor {}

    impl Bench for SimplifyFactor {
        fn name(&self) -> &str {
            &"math_simplify_factor"
        }

        fn run_egg(&self) {
            let start_expr = &"(* (+ x 3) (+ x 1))";
            let end_expr = &"(+ (+ (* x x) (* 4 x)) 3)";
            let _runner = run_and_check(start_expr, end_expr, default_runner(), rules(), true);
        }

        fn egglog_text(&self) -> Option<String> {
            let mut src = crate::get_text(&"math_full")?;
            src.push_str(
                &r#"
            (define start-expr (Mul (Add (Var "x") (Const (rational 3 1)))
                                    (Add (Var "x") (Const (rational 1 1)))))
            (run 8)
            (define end-expr (Add (Add (Mul (Var "x") (Var "x"))
                                    (Mul (Const (rational 4 1)) (Var "x")))
                                    (Const (rational 3 1))))
            (check (= start-expr end-expr))
            "#,
            );
            Some(src)
        }
    }
}

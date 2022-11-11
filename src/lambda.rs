use crate::*;

pub mod lambda_egg_src {
    use egg::{rewrite as rw, *};
    use std::collections::HashSet;

    define_language! {
        pub enum Lambda {
            Bool(bool),
            Num(i32),

            "var" = Var(Id),

            "+" = Add([Id; 2]),
            "=" = Eq([Id; 2]),

            "app" = App([Id; 2]),
            "lam" = Lambda([Id; 2]),
            "let" = Let([Id; 3]),
            "fix" = Fix([Id; 2]),

            "if" = If([Id; 3]),
            "fresh" = Fresh([Id; 1]),

            Symbol(egg::Symbol),
        }
    }

    impl Lambda {
        fn num(&self) -> Option<i32> {
            match self {
                Lambda::Num(n) => Some(*n),
                _ => None,
            }
        }
    }

    type EGraph = egg::EGraph<Lambda, LambdaAnalysis>;

    #[derive(Default)]
    pub struct LambdaAnalysis;

    #[derive(Debug)]
    pub struct Data {
        free: HashSet<Id>,
        constant: Option<(Lambda, PatternAst<Lambda>)>,
    }

    fn eval(egraph: &EGraph, enode: &Lambda) -> Option<(Lambda, PatternAst<Lambda>)> {
        let x = |i: &Id| egraph[*i].data.constant.as_ref().map(|c| &c.0);
        match enode {
            Lambda::Num(n) => Some((enode.clone(), format!("{}", n).parse().unwrap())),
            Lambda::Bool(b) => Some((enode.clone(), format!("{}", b).parse().unwrap())),
            Lambda::Add([a, b]) => Some((
                Lambda::Num(x(a)?.num()? + x(b)?.num()?),
                format!("(+ {} {})", x(a)?, x(b)?).parse().unwrap(),
            )),
            Lambda::Eq([a, b]) => Some((
                Lambda::Bool(x(a)? == x(b)?),
                format!("(= {} {})", x(a)?, x(b)?).parse().unwrap(),
            )),
            _ => None,
        }
    }

    impl Analysis<Lambda> for LambdaAnalysis {
        type Data = Data;
        fn merge(&mut self, to: &mut Data, from: Data) -> DidMerge {
            let before_len = to.free.len();
            // to.free.extend(from.free);
            to.free.retain(|i| from.free.contains(i));
            // compare lengths to see if I changed to or from
            DidMerge(
                before_len != to.free.len(),
                to.free.len() != from.free.len(),
            ) | merge_option(&mut to.constant, from.constant, |a, b| {
                assert_eq!(a.0, b.0, "Merged non-equal constants");
                DidMerge(false, false)
            })
        }

        fn make(egraph: &EGraph, enode: &Lambda) -> Data {
            let f = |i: &Id| egraph[*i].data.free.iter().cloned();
            let mut free = HashSet::default();
            match enode {
                Lambda::Var(v) => {
                    free.insert(*v);
                }
                Lambda::Let([v, a, b]) => {
                    free.extend(f(b));
                    free.remove(v);
                    free.extend(f(a));
                }
                Lambda::Lambda([v, a]) | Lambda::Fix([v, a]) => {
                    free.extend(f(a));
                    free.remove(v);
                }
                _ => enode.for_each(|c| free.extend(&egraph[c].data.free)),
            }
            let constant = eval(egraph, enode);
            Data { constant, free }
        }

        fn modify(egraph: &mut EGraph, id: Id) {
            if let Some(c) = egraph[id].data.constant.clone() {
                if egraph.are_explanations_enabled() {
                    egraph.union_instantiations(
                        &c.0.to_string().parse().unwrap(),
                        &c.1,
                        &Default::default(),
                        "analysis".to_string(),
                    );
                } else {
                    let const_id = egraph.add(c.0);
                    egraph.union(id, const_id);
                }
            }
        }
    }

    fn var(s: &str) -> Var {
        s.parse().unwrap()
    }

    fn is_not_same_var(v1: Var, v2: Var) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
        move |egraph, _, subst| egraph.find(subst[v1]) != egraph.find(subst[v2])
    }

    fn is_const(v: Var) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
        move |egraph, _, subst| egraph[subst[v]].data.constant.is_some()
    }

    pub fn rules() -> Vec<Rewrite<Lambda, LambdaAnalysis>> {
        vec![
            // open term rules
            rw!("if-true";  "(if  true ?then ?else)" => "?then"),
            rw!("if-false"; "(if false ?then ?else)" => "?else"),
            rw!("if-elim"; "(if (= (var ?x) ?e) ?then ?else)" => "?else"
            if ConditionEqual::parse("(let ?x ?e ?then)", "(let ?x ?e ?else)")),
            rw!("add-comm";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
            rw!("add-assoc"; "(+ (+ ?a ?b) ?c)" => "(+ ?a (+ ?b ?c))"),
            rw!("eq-comm";   "(= ?a ?b)"        => "(= ?b ?a)"),
            // subst rules
            rw!("fix";      "(fix ?v ?e)"             => "(let ?v (fix ?v ?e) ?e)"),
            rw!("beta";     "(app (lam ?v ?body) ?e)" => "(let ?v ?e ?body)"),
            rw!("let-app";  "(let ?v ?e (app ?a ?b))" => "(app (let ?v ?e ?a) (let ?v ?e ?b))"),
            rw!("let-add";  "(let ?v ?e (+   ?a ?b))" => "(+   (let ?v ?e ?a) (let ?v ?e ?b))"),
            rw!("let-eq";   "(let ?v ?e (=   ?a ?b))" => "(=   (let ?v ?e ?a) (let ?v ?e ?b))"),
            rw!("let-const";
            "(let ?v ?e ?c)" => "?c" if is_const(var("?c"))),
            rw!("let-if";
                "(let ?v ?e (if ?cond ?then ?else))" =>
                "(if (let ?v ?e ?cond) (let ?v ?e ?then) (let ?v ?e ?else))"
            ),
            rw!("let-var-same"; "(let ?v1 ?e (var ?v1))" => "?e"),
            rw!("let-var-diff"; "(let ?v1 ?e (var ?v2))" => "(var ?v2)"
            if is_not_same_var(var("?v1"), var("?v2"))),
            rw!("let-lam-same"; "(let ?v1 ?e (lam ?v1 ?body))" => "(lam ?v1 ?body)"),
            rw!("let-lam-diff";
            "(let ?v1 ?e (lam ?v2 ?body))" =>
            { CaptureAvoid {
                v2: var("?v2"), e: var("?e"),
                if_not_free: "(lam ?v2 (let ?v1 ?e ?body))".parse().unwrap(),
                if_free: "(lam (fresh (let ?v1 ?e (lam ?v2 ?body))) (let ?v1 ?e (let ?v2 (var (fresh (let ?v1 ?e (lam ?v2 ?body)))) ?body)))".parse().unwrap(),
            }}
            if is_not_same_var(var("?v1"), var("?v2"))),
        ]
    }

    struct CaptureAvoid {
        v2: Var,
        e: Var,
        if_not_free: Pattern<Lambda>,
        if_free: Pattern<Lambda>,
    }

    impl Applier<Lambda, LambdaAnalysis> for CaptureAvoid {
        fn apply_one(
            &self,
            egraph: &mut EGraph,
            eclass: Id,
            subst: &Subst,
            searcher_ast: Option<&PatternAst<Lambda>>,
            rule_name: Symbol,
        ) -> Vec<Id> {
            let e = subst[self.e];
            let v2 = subst[self.v2];
            let v2_free_in_e = egraph[e].data.free.contains(&v2);
            if v2_free_in_e {
                self.if_free
                    .apply_one(egraph, eclass, &subst, searcher_ast, rule_name)
            } else {
                self.if_not_free
                    .apply_one(egraph, eclass, subst, searcher_ast, rule_name)
            }
        }
    }
}

pub mod run_n {

    use super::lambda_egg_src::rules;
    use super::*;

    pub(crate) fn new(n: usize) -> Box<dyn Benchmark> {
        Box::new(RunN { n })
    }
    pub struct RunN {
        n: usize,
    }

    impl Benchmark for RunN {
        fn name(&self) -> String {
            format!("lamb-run-{}", self.n)
        }

        fn run_egg(&self) -> usize {
            let start_exprs = vec![
                "(let zeroone (lam x
                    (if (= (var x) 0)
                        0
                        1))
                    (+ (app (var zeroone) 0)
                    (app (var zeroone) 10)))",
                "(let compose (lam f (lam g (lam x (app (var f)
                                                (app (var g) (var x))))))
                (let repeat (fix repeat (lam fun (lam n
                    (if (= (var n) 0)
                        (lam i (var i))
                        (app (app (var compose) (var fun))
                            (app (app (var repeat)
                                    (var fun))
                                (+ (var n) -1)))))))
                (let add1 (lam y (+ (var y) 1))
                (app (app (var repeat)
                        (var add1))
                    2))))",
                "(let fib (fix fib (lam n
                    (if (= (var n) 0)
                        0
                    (if (= (var n) 1)
                        1
                    (+ (app (var fib)
                            (+ (var n) -1))
                        (app (var fib)
                            (+ (var n) -2)))))))
                    (app (var fib) 4))",
            ];
            let mut runner = default_runner();
            runner = runner
                .with_scheduler(egg::BackoffScheduler::default().with_initial_match_limit(1000))
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
            let mut src = crate::get_text(&"lambda_full")?;
            src.push_str(&format!(
                r#"

                (Let (V "zeroone") (Lam (V "x")
                    (If (Eq (Var (V "x")) (Val (Num 0)))
                        (Val (Num 0))
                        (Val (Num 1))))
                    (Add (App (Var (V "zeroone")) (Val (Num 0)))
                    (App (Var (V "zeroone")) (Val (Num 10)))))
                (Let (V "compose") (Lam (V "f") (Lam (V "g") (Lam (V "x") (App (Var (V "f"))
                                                (App (Var (V "g")) (Var (V "x")))))))
                (Let (V "repeat") (Fix (V "repeat") (Lam (V "fun") (Lam (V "n")
                    (If (Eq (Var (V "n")) (Val (Num 0)))
                        (Lam (V "i") (Var (V "i")))
                        (App (App (Var (V "compose")) (Var (V "fun")))
                            (App (App (Var (V "repeat"))
                                    (Var (V "fun")))
                                (Add (Var (V "n")) (Val (Num -1)))))))))
                (Let (V "add1") (Lam (V "y") (Add (Var (V "y")) (Val (Num 1))))
                (App (App (Var (V "repeat"))
                        (Var (V "add1")))
                    (Val (Num 2))))))
                (Let (V "fib") (Fix (V "fib") (Lam (V "n")
                    (If (Eq (Var (V "n")) (Val (Num 0)))
                        (Val (Num 0))
                    (If (Eq (Var (V "n")) (Val (Num 1)))
                        (Val (Num 1))
                    (Add (App (Var (V "fib"))
                            (Add (Var (V "n")) (Val (Num -1))))
                        (App (Var (V "fib"))
                            (Add (Var (V "n")) (Val (Num -2)))))))))
                    (App (Var (V "fib")) (Val (Num 4))))
                (run {})
                (print-size V)
                (print-size From)
                (print-size Val)
                (print-size Var)
                (print-size Add)
                (print-size Eq)
                (print-size App)
                (print-size Lam)
                (print-size Let)
                (print-size Fix)
                (print-size If)
                "#,
                self.n
            ));
            Some(src)
        }

        fn run_egglog(&mut self) -> usize {
            let mut egraph = egg_smol::EGraph::default();
            egraph.match_limit = 1000;
            self.run_egglog_with_engine(egraph)
        }
        fn run_egglognaive(&mut self) -> usize {
            let mut egraph = egg_smol::EGraph::default();
            egraph.match_limit = 1000;
            egraph.seminaive = false;
            self.run_egglog_with_engine(egraph)
        }
    }
}

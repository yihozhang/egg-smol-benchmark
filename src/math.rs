use crate::Bench;
use egg::*;
use egg::rewrite as rw;
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
    pub struct AC;

    pub fn new() -> AC {
        AC
    }

    impl AC {

        fn rewrites(&self) -> Vec<Rewrite> {
            vec![        
                rw!("comm-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
                rw!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
            ]
        }

        fn egglog_text(&self) -> &str {
            &"
            (datatype Math
              (Var i64)
              (Add Math Math)
            )
            
            (define start (Add (Var 1) (Add (Var 2) (Add (Var 3) (Add (Var 4) 
                          (Add (Var 5) (Add (Var 6) (Add (Var 7) (Var 8)))))))))
            
            (rewrite (Add x y) (Add y x))
            (rewrite (Add x (Add y z)) (Add (Add x y) z))
            
            (run 8)
            
            (define end (Add (Var 8) (Add (Var 7) (Add (Var 6) (Add (Var 5) 
                        (Add (Var 4) (Add (Var 3) (Add (Var 2) (Var 1)))))))))
            (check (= start end))"
        }
    }

    impl Bench for AC {
        fn name(&self) -> std::string::String {
            "assoc-comm".into()
        }
        fn run_egg(&self) {
            let start_expr = "(+ 1 (+ 2 (+ 3 (+ 4 (+ 5 (+ 6 (+ 7 8)))))))"
                .parse()
                .unwrap();
            let end_expr = "(+ 8 (+ 7 (+ 6 (+ 5 (+ 4 (+ 3 (+ 2 1)))))))"
                .parse()
                .unwrap();
            let runner = Runner::default()
                .with_iter_limit(8)
                .with_scheduler(SimpleScheduler)
                .with_expr(&start_expr)
                .run(&self.rewrites());
            let egraph = &runner.egraph;
            assert!(egraph.equivs(&start_expr, &end_expr).len() == 1);
        }

        fn run_egglog(&self) {
            let mut egraph = egg_smol::EGraph::default();
            let result = egraph.parse_and_run_program(self.egglog_text());
            assert!(result.is_ok());
        }
    }
}

// // TODO: Support small rational in egglog
// // so that rational operations aren't the bottleneck

// define_language! {
//     pub enum Math {
//         "d" = Diff([Id; 2]),
//         "i" = Integral([Id; 2]),

//         "+" = Add([Id; 2]),
//         "-" = Sub([Id; 2]),
//         "*" = Mul([Id; 2]),
//         "/" = Div([Id; 2]),
//         "pow" = Pow([Id; 2]),
//         "ln" = Ln(Id),
//         "sqrt" = Sqrt(Id),

//         "sin" = Sin(Id),
//         "cos" = Cos(Id),

//         Constant(Constant),
//         Symbol(Symbol),
//     }
// }

// // You could use egg::AstSize, but this is useful for debugging, since
// // it will really try to get rid of the Diff operator
// pub struct MathCostFn;
// impl egg::CostFunction<Math> for MathCostFn {
//     type Cost = usize;
//     fn cost<C>(&mut self, enode: &Math, mut costs: C) -> Self::Cost
//     where
//         C: FnMut(Id) -> Self::Cost,
//     {
//         let op_cost = match enode {
//             Math::Diff(..) => 100,
//             Math::Integral(..) => 100,
//             _ => 1,
//         };
//         enode.fold(op_cost, |sum, i| sum + costs(i))
//     }
// }

// #[derive(Default)]
// pub struct ConstantFold;
// impl Analysis<Math> for ConstantFold {
//     type Data = Option<Constant>;

//     fn make(egraph: &EGraph, enode: &Math) -> Self::Data {
//         let x = |i: &Id| egraph[*i].data.as_ref().map(|d| d);
//         Some(match enode {
//             Math::Constant(c) => *c,
//             Math::Add([a, b]) => x(a)? + x(b)?,
//             Math::Sub([a, b]) => x(a)? - x(b)?,
//             Math::Mul([a, b]) => x(a)? * x(b)?,
//             Math::Div([a, b]) if x(b) != Some(&BigRational::zero()) => x(a)? / x(b)?,
//             _ => return None,
//         })
//     }

//     fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
//         merge_option(to, from, |a, b| {
//             assert_eq!(a, b, "Merged non-equal constants");
//             DidMerge(false, false)
//         })
//     }

//     fn modify(egraph: &mut EGraph, id: Id) {
//         let class = egraph[id].clone();
//         if let Some(c) = class.data {
//             let added = egraph.add(Math::Constant(c));
//             egraph.union(id, added);
//             // to not prune, comment this out
//             egraph[id].nodes.retain(|n| n.is_leaf());

//             #[cfg(debug_assertions)]
//             egraph[id].assert_unique_leaves();
//         }
//     }
// }

// fn is_const_or_distinct_var(v: &str, w: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
//     let v = v.parse().unwrap();
//     let w = w.parse().unwrap();
//     move |egraph, _, subst| {
//         egraph.find(subst[v]) != egraph.find(subst[w])
//             && (egraph[subst[v]].data.is_some()
//                 || egraph[subst[v]]
//                     .nodes
//                     .iter()
//                     .any(|n| matches!(n, Math::Symbol(..))))
//     }
// }

// fn is_const(var: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
//     let var = var.parse().unwrap();
//     move |egraph, _, subst| egraph[subst[var]].data.is_some()
// }

// fn is_sym(var: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
//     let var = var.parse().unwrap();
//     move |egraph, _, subst| {
//         egraph[subst[var]]
//             .nodes
//             .iter()
//             .any(|n| matches!(n, Math::Symbol(..)))
//     }
// }

// fn is_not_zero(var: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
//     let var = var.parse().unwrap();
//     move |egraph, _, subst| {
//         if let Some(n) = &egraph[subst[var]].data {
//             !n.is_zero()
//         } else {
//             true
//         }
//     }
// }

// #[rustfmt::skip]
// pub fn rules() -> Vec<Rewrite> { vec![
//     rw!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
//     rw!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
//     rw!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
//     rw!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),

//     rw!("sub-canon"; "(- ?a ?b)" => "(+ ?a (* -1 ?b))"),
//     rw!("div-canon"; "(/ ?a ?b)" => "(* ?a (pow ?b -1))" if is_not_zero("?b")),
//     // rw!("canon-sub"; "(+ ?a (* -1 ?b))"   => "(- ?a ?b)"),
//     // rw!("canon-div"; "(* ?a (pow ?b -1))" => "(/ ?a ?b)" if is_not_zero("?b")),

//     rw!("zero-add"; "(+ ?a 0)" => "?a"),
//     rw!("zero-mul"; "(* ?a 0)" => "0"),
//     rw!("one-mul";  "(* ?a 1)" => "?a"),

//     rw!("add-zero"; "?a" => "(+ ?a 0)"),
//     rw!("mul-one";  "?a" => "(* ?a 1)"),

//     rw!("cancel-sub"; "(- ?a ?a)" => "0"),
//     rw!("cancel-div"; "(/ ?a ?a)" => "1" if is_not_zero("?a")),

//     rw!("distribute"; "(* ?a (+ ?b ?c))"        => "(+ (* ?a ?b) (* ?a ?c))"),
//     rw!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),

//     rw!("pow-mul"; "(* (pow ?a ?b) (pow ?a ?c))" => "(pow ?a (+ ?b ?c))"),
//     rw!("pow0"; "(pow ?x 0)" => "1"
//         if is_not_zero("?x")),
//     rw!("pow1"; "(pow ?x 1)" => "?x"),
//     rw!("pow2"; "(pow ?x 2)" => "(* ?x ?x)"),
//     rw!("pow-recip"; "(pow ?x -1)" => "(/ 1 ?x)"
//         if is_not_zero("?x")),
//     rw!("recip-mul-div"; "(* ?x (/ 1 ?x))" => "1" if is_not_zero("?x")),

//     rw!("d-variable"; "(d ?x ?x)" => "1" if is_sym("?x")),
//     rw!("d-constant"; "(d ?x ?c)" => "0" if is_sym("?x") if is_const_or_distinct_var("?c", "?x")),

//     rw!("d-add"; "(d ?x (+ ?a ?b))" => "(+ (d ?x ?a) (d ?x ?b))"),
//     rw!("d-mul"; "(d ?x (* ?a ?b))" => "(+ (* ?a (d ?x ?b)) (* ?b (d ?x ?a)))"),

//     rw!("d-sin"; "(d ?x (sin ?x))" => "(cos ?x)"),
//     rw!("d-cos"; "(d ?x (cos ?x))" => "(* -1 (sin ?x))"),

//     rw!("d-ln"; "(d ?x (ln ?x))" => "(/ 1 ?x)" if is_not_zero("?x")),

//     rw!("d-power";
//         "(d ?x (pow ?f ?g))" =>
//         "(* (pow ?f ?g)
//             (+ (* (d ?x ?f)
//                   (/ ?g ?f))
//                (* (d ?x ?g)
//                   (ln ?f))))"
//         if is_not_zero("?f")
//         if is_not_zero("?g")
//     ),

//     rw!("i-one"; "(i 1 ?x)" => "?x"),
//     rw!("i-power-const"; "(i (pow ?x ?c) ?x)" =>
//         "(/ (pow ?x (+ ?c 1)) (+ ?c 1))" if is_const("?c")),
//     rw!("i-cos"; "(i (cos ?x) ?x)" => "(sin ?x)"),
//     rw!("i-sin"; "(i (sin ?x) ?x)" => "(* -1 (cos ?x))"),
//     rw!("i-sum"; "(i (+ ?f ?g) ?x)" => "(+ (i ?f ?x) (i ?g ?x))"),
//     rw!("i-dif"; "(i (- ?f ?g) ?x)" => "(- (i ?f ?x) (i ?g ?x))"),
//     rw!("i-parts"; "(i (* ?a ?b) ?x)" =>
//         "(- (* ?a (i ?b ?x)) (i (* (d ?x ?a) (i ?b ?x)) ?x))"),
// ]}

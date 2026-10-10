//! Dyadic verbs in the grammar: `= < > <> <= >=` evaluate, `# , !` reach their kernels.
use oxidedb::language::ast::Verb;
use oxidedb::language::ops;
use oxidedb::types::atom::Atom;
use oxidedb::types::column::Column;
use oxidedb::types::sym::Sym;
use oxidedb::types::value::Value;
use oxidedb::{Interpreter, QError};
use proptest::prelude::*;
use std::rc::Rc;

fn eval(src: &str) -> Result<Value, QError> {
    Interpreter::new()
        .eval_line(src)
        .map(|v| v.expect("a value"))
}

fn show(src: &str) -> String {
    eval(src)
        .unwrap_or_else(|e| panic!("{src}: {e}"))
        .to_string()
}

fn err(src: &str) -> String {
    eval(src).expect_err(src).to_string()
}

fn bools(v: &[bool]) -> Value {
    Value::Vector(Rc::new(Column::Bool(v.to_vec())))
}

fn long(n: i64) -> Value {
    Value::Atom(Atom::Integer(n))
}

#[test]
fn verbs_each_comparison_maps_to_its_own_verb() {
    // 1 vs 2 (less), 2 vs 2 (equal), 2 vs 1 (greater): every verb answers differently.
    let table = [
        ("=", "010b"),
        ("<>", "101b"),
        ("<", "100b"),
        ("<=", "110b"),
        (">", "001b"),
        (">=", "011b"),
    ];
    for (verb, want) in table {
        assert_eq!(show(&format!("1 2 2 {verb} 2 2 1")), want, "{verb}");
        let atom = if want.starts_with('1') { "1b" } else { "0b" };
        assert_eq!(show(&format!("1 {verb} 2")), atom, "{verb}");
    }
}

#[test]
fn verbs_vector_results_are_boolean_vectors() {
    assert_eq!(eval("1 2 3 = 1 5 3"), Ok(bools(&[true, false, true])));
    assert_eq!(show("1 2 3 = 1 5 3"), "101b");
    assert_eq!(eval("1 2 3 = 1 5 3").unwrap().type_code(), 1);
    assert_eq!(show("10 20 30 > 15"), "011b");
    assert_eq!(show("15 < 10 20 30"), "011b");
}

#[test]
fn verbs_right_to_left_no_precedence() {
    assert_eq!(show("1 < 2 + 3"), "1b");
    assert_eq!(show("2 + 3 > 1"), "3");
    assert_eq!(show("1 + 2 < 4"), "2");
    assert_eq!(show("1 2 3 = 1 2 3 = 1 2 3"), "100b");
    assert_eq!(show("0 = 1 = 1"), "0b");
    assert_eq!(show("(1 < 2) + 1"), "2");
    assert_eq!(show("10 * 2 = 20"), "0");
}

#[test]
fn verbs_negative_literals_after_each_verb() {
    assert_eq!(show("1<-2"), "0b");
    assert_eq!(show("2=-1"), "0b");
    assert_eq!(show("-1=-1"), "1b");
    assert_eq!(show("1>-2"), "1b");
    assert_eq!(show("1>=-2"), "1b");
    assert_eq!(show("-2<=-2"), "1b");
    assert_eq!(show("1<>-1"), "1b");
    assert_eq!(show("1 < -2"), "0b");
    assert_eq!(show("1 2 3 > -1 5 -3"), "101b");
    // after # , ! the negative literal still lexes as a number
    assert_eq!(show("1#-2"), ",-2");
    assert_eq!(show("1,-2"), "1 -2");
    assert_eq!(show("-2#1 2 3"), "2 3");
    assert_eq!(err("1!-2"), "'nyi: !");
}

#[test]
fn verbs_comments_and_whitespace() {
    assert_eq!(show("1 2 3 = 1 5 3 / c"), "101b");
    assert_eq!(show("1 2 3=1 5 3"), "101b");
    assert_eq!(show("1<=2 /c"), "1b");
    assert_eq!(show("  1   <  2  "), "1b");
}

#[test]
fn verbs_nulls() {
    assert_eq!(show("0N=0N"), "1b");
    assert_eq!(show("0N<1"), "1b");
    assert_eq!(show("1<0N"), "0b");
    assert_eq!(show("0n=0n"), "1b");
    assert_eq!(show("0N=0n"), "1b");
    assert_eq!(show("0n<-0w"), "1b");
    assert_eq!(show("1 0N 3 = 1 0N 4"), "110b");
    assert_eq!(show("0N 1 > 0N"), "01b");
}

#[test]
fn verbs_strings_and_symbols() {
    assert_eq!(show("\"abc\"=\"abd\""), "110b");
    assert_eq!(show("\"abc\"=\"b\""), "010b");
    assert_eq!(show("`a`b=`a`c"), "10b");
    assert_eq!(show("`a<`b"), "1b");
    assert_eq!(show("`a=`a"), "1b");
    assert_eq!(show("\"a\"<\"b\""), "1b");
    assert_eq!(show("`a`b`c<>`a`x`c"), "010b");
    // Deliberate O deviation: q compares a character with a number by code.
    assert_eq!(eval("\"abc\"=1"), Err(QError::Type));
    assert_eq!(eval("\"a\"<1"), Err(QError::Type));
    assert_eq!(eval("`a=1"), Err(QError::Type));
    assert_eq!(eval("`a=\"a\""), Err(QError::Type));
}

#[test]
fn verbs_length_mismatch() {
    assert_eq!(eval("1 2 3 = 1 2"), Err(QError::Length));
    assert_eq!(eval("1 2 < 1 2 3"), Err(QError::Length));
    assert_eq!(eval("\"ab\"=\"abc\""), Err(QError::Length));
}

#[test]
fn verbs_assignment_of_results() {
    let mut i = Interpreter::new();
    assert_eq!(
        i.eval_line("v:1 2 3").unwrap().unwrap().to_string(),
        "1 2 3"
    );
    assert_eq!(i.eval_line("m:v>1").unwrap().unwrap().to_string(), "011b");
    assert_eq!(i.eval_line("m").unwrap().unwrap().to_string(), "011b");
    assert_eq!(i.eval_line("m=0 1 1").unwrap().unwrap().to_string(), "111b");
}

#[test]
fn verbs_right_operand_first() {
    // x+x:2 is 4 and the same holds under a comparison: the assignment runs first.
    let mut i = Interpreter::new();
    i.eval_line("x:1").unwrap();
    assert_eq!(
        i.eval_line("x<x:2").unwrap().unwrap(),
        Value::Atom(Atom::Boolean(false))
    );
    assert_eq!(i.eval_line("x").unwrap().unwrap(), long(2));
    assert_eq!(i.eval_line("x=x:5").unwrap().unwrap().to_string(), "1b");
}

#[test]
fn verbs_undefined_name_and_dangling_verb() {
    assert_eq!(eval("zz<1"), Err(QError::Undefined("zz".into())));
    assert_eq!(eval("1<"), Err(QError::parse("unexpected end of input")));
    assert_eq!(eval("1 = = 2"), Err(QError::Nyi("=".into())));
}

#[test]
fn verbs_monadic_use_is_nyi_with_the_verb() {
    for (src, verb) in [
        ("=1", "="),
        ("<1", "<"),
        (">1", ">"),
        ("<>1", "<>"),
        ("<=1", "<="),
        (">=1", ">="),
        ("#1 2", "#"),
        (",1", ","),
        ("!3", "!"),
        ("1 + #2", "#"),
    ] {
        assert_eq!(eval(src), Err(QError::Nyi(verb.into())), "{src}");
    }
}

#[test]
fn verbs_take_join_key_reach_their_kernels() {
    // Whatever the kernel answers today, the grammar must hand it exactly these operands.
    let l = Value::Vector(Rc::new(Column::Long(vec![1, 2, 3])));
    let two = long(2);
    for (src, verb, a, b) in [
        ("1 2 3 # 2", Verb::Take, &l, &two),
        ("2 , 1 2 3", Verb::Join, &two, &l),
        ("2 ! 1 2 3", Verb::Key, &two, &l),
        ("1 2 3 , 2", Verb::Join, &l, &two),
        ("2 # 1 2 3", Verb::Take, &two, &l),
    ] {
        assert_eq!(eval(src), ops::dyad(verb, a, b), "{src}");
    }
    // right to left through a chain: 2 # (1 2 3 + 1)
    let plus = ops::dyad(Verb::Add, &l, &long(1)).unwrap();
    assert_eq!(eval("2 # 1 2 3 + 1"), ops::dyad(Verb::Take, &two, &plus));
    // and `#` result feeds a comparison on its left
    let taken = ops::dyad(Verb::Take, &two, &l);
    if let Ok(t) = taken {
        assert_eq!(
            eval("1 2 = 2 # 1 2 3"),
            ops::dyad(
                Verb::Equal,
                &Value::Vector(Rc::new(Column::Long(vec![1, 2]))),
                &t
            )
        );
    }
}

#[test]
fn verbs_take_join_key_real_results() {
    assert_eq!(show("2#1 2 3"), "1 2");
    assert_eq!(show("5#1 2"), "1 2 1 2 1");
    assert_eq!(show("1 2,3 4"), "1 2 3 4");
    assert_eq!(show("1 2,3"), "1 2 3");
    assert_eq!(show("\"ab\",\"c\""), "\"abc\"");
    // right to left: 2 # (1 2 3 + 1), and a join feeding a comparison
    assert_eq!(show("2#1 2 3 + 1"), "2 3");
    assert_eq!(show("(1 2,3) = 1 2 4"), "110b");
    assert_eq!(show("1 2,3 = 3"), "1\n2\n1b");
    // each verb is wired to its own token: an undefined right operand is reported by the evaluator
    for src in ["1#u", "1,u", "1!u"] {
        assert_eq!(err(src), "'u (Undefined variable)", "{src}");
    }
    assert_eq!(err("1!2"), "'nyi: !");
    assert_eq!(err("1 2!3 4"), "'nyi: !");
}

#[test]
fn verbs_other_punctuation_is_still_nyi() {
    assert_eq!(eval("1 $ 2"), Err(QError::Nyi("$".into())));
    assert_eq!(eval("1 @ 2"), Err(QError::Nyi("@".into())));
    assert_eq!(eval("1 2 3/2"), Err(QError::Nyi("adverb '/'".into())));
}

#[test]
fn verbs_chain_limit_holds_for_new_verbs() {
    let n = 200_000;
    let src = format!("{}1", "1=".repeat(n));
    let e = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || eval(&src).map(|v| v.to_string()).map_err(|e| e.to_string()))
        .unwrap()
        .join()
        .unwrap();
    assert!(e.unwrap_err().starts_with("'parse: expression too long"));
    // mixed verbs count toward the same cap
    let mixed = format!("{}1", "1<1+".repeat(1500));
    let err = eval(&mixed).unwrap_err().to_string();
    assert!(err.starts_with("'parse: expression too long"), "{err}");
}

#[test]
fn verbs_symbol_and_char_atoms_compare_through_variables() {
    let mut i = Interpreter::new();
    i.set("s", Value::Atom(Atom::Symbol(Sym::intern("a"))));
    assert_eq!(i.eval_line("s=`a").unwrap().unwrap().to_string(), "1b");
}

proptest! {
    #[test]
    fn verbs_atoms_match_rust_comparisons(a in -1000i64..1000, b in -1000i64..1000) {
        let t = |s: String| eval(&s).unwrap() == Value::Atom(Atom::Boolean(true));
        prop_assert_eq!(t(format!("{a} = {b}")), a == b);
        prop_assert_eq!(t(format!("{a} <> {b}")), a != b);
        prop_assert_eq!(t(format!("{a} < {b}")), a < b);
        prop_assert_eq!(t(format!("{a} <= {b}")), a <= b);
        prop_assert_eq!(t(format!("{a} > {b}")), a > b);
        prop_assert_eq!(t(format!("{a} >= {b}")), a >= b);
    }

    #[test]
    fn verbs_vectors_match_pairwise(v in proptest::collection::vec((-5i64..5, -5i64..5), 2..12)) {
        let (x, y): (Vec<i64>, Vec<i64>) = v.into_iter().unzip();
        let text = |v: &[i64]| v.iter().map(i64::to_string).collect::<Vec<_>>().join(" ");
        let want: Vec<bool> = x.iter().zip(&y).map(|(a, b)| a < b).collect();
        prop_assert_eq!(eval(&format!("{} < {}", text(&x), text(&y))), Ok(bools(&want)));
    }
}

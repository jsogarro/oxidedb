//! Vector literals: `1 2 3`, `1 2.5 3`, `101b`, `` `a`b ``, `"abc"`.
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

fn vec_of(c: Column) -> Value {
    Value::Vector(Rc::new(c))
}

fn longs(v: &[i64]) -> Value {
    vec_of(Column::Long(v.to_vec()))
}

fn floats(v: &[f64]) -> Value {
    vec_of(Column::Float(v.to_vec()))
}

fn syms(v: &[&str]) -> Value {
    vec_of(Column::Sym(v.iter().map(|s| Sym::intern(s)).collect()))
}

const N: i64 = i64::MIN;

#[test]
fn vec_lit_long() {
    assert_eq!(eval("1 2 3"), Ok(longs(&[1, 2, 3])));
    assert_eq!(show("1 2 3"), "1 2 3");
    assert_eq!(eval("1 2 3").unwrap().type_code(), 7);
    // a single number is still an atom
    assert_eq!(eval("2.5"), Ok(Value::Atom(Atom::Float(2.5))));
    assert_eq!(eval("5"), Ok(Value::Atom(Atom::Integer(5))));
}

#[test]
fn vec_lit_float_promotes() {
    assert_eq!(eval("1 2.5 3"), Ok(floats(&[1.0, 2.5, 3.0])));
    assert_eq!(eval("1 2 3f"), Ok(floats(&[1.0, 2.0, 3.0])));
    assert_eq!(eval("1.5 2"), Ok(floats(&[1.5, 2.0])));
    assert_eq!(eval("1 2.5 3").unwrap().type_code(), 9);
    assert_eq!(show("1 2.5 3"), "1 2.5 3");
    assert_eq!(show("1 2 3f"), "1 2 3f");
}

#[test]
fn vec_lit_negative() {
    assert_eq!(eval("1 -2 3"), Ok(longs(&[1, -2, 3])));
    assert_eq!(eval("-1 2 3"), Ok(longs(&[-1, 2, 3])));
    assert_eq!(eval("1 -2"), Ok(longs(&[1, -2])));
    assert_eq!(eval("1 -2.5"), Ok(floats(&[1.0, -2.5])));
    assert_eq!(eval("1 - 2"), Ok(Value::Atom(Atom::Integer(-1))));
    assert_eq!(eval("1-2"), Ok(Value::Atom(Atom::Integer(-1))));
    assert_eq!(show("1 -2 3"), "1 -2 3");
}

#[test]
fn vec_lit_nulls() {
    assert_eq!(eval("1 0N 3"), Ok(longs(&[1, N, 3])));
    assert_eq!(eval("0N 0N"), Ok(longs(&[N, N])));
    assert_eq!(show("1 0N 3"), "1 0N 3");
    // a float anywhere makes the long null the float null
    let nan = f64::NAN;
    assert_eq!(eval("1 0n"), Ok(floats(&[1.0, nan])));
    assert_eq!(eval("0N 0n"), Ok(floats(&[nan, nan])));
    assert_eq!(eval("0N 1.5"), Ok(floats(&[nan, 1.5])));
    assert_eq!(show("0N 0n"), "0n 0n");
    assert_eq!(eval("1 0w"), Ok(floats(&[1.0, f64::INFINITY])));
    assert_eq!(eval("-0w 1"), Ok(floats(&[f64::NEG_INFINITY, 1.0])));
}

#[test]
fn vec_lit_bool() {
    assert_eq!(
        eval("101b"),
        Ok(vec_of(Column::Bool(vec![true, false, true])))
    );
    assert_eq!(eval("00b"), Ok(vec_of(Column::Bool(vec![false, false]))));
    // bit order is not palindromic
    assert_eq!(
        eval("110b"),
        Ok(vec_of(Column::Bool(vec![true, true, false])))
    );
    assert_eq!(
        eval("100b"),
        Ok(vec_of(Column::Bool(vec![true, false, false])))
    );
    assert_eq!(show("101b"), "101b");
    assert_eq!(eval("101b").unwrap().type_code(), 1);
    // a single boolean stays an atom
    assert_eq!(eval("1b"), Ok(Value::Atom(Atom::Boolean(true))));
}

#[test]
fn vec_lit_booleans_do_not_join_runs() {
    // Juxtaposed nouns are application, whatever their kind: an atom cannot be
    // applied, a vector is indexed (a boolean index reads as 0 or 1).
    for src in [
        "1 1b", "1b 1", "1b 0b", "1b 2 3", "1.5 1b", "1b 2.5", "1 101b",
    ] {
        assert_eq!(eval(src), Err(QError::Type), "{src}");
    }
    assert_eq!(show("1 2 1b"), "2");
    assert_eq!(show("101b 010b"), "101b");
}

#[test]
fn vec_lit_symbols() {
    assert_eq!(eval("`a`b`c"), Ok(syms(&["a", "b", "c"])));
    assert_eq!(show("`a`b`c"), "`a`b`c");
    assert_eq!(eval("`a`b").unwrap().type_code(), 11);
}

#[test]
fn vec_lit_symbol_atom() {
    assert_eq!(eval("`a"), Ok(Value::Atom(Atom::Symbol(Sym::intern("a")))));
    assert_eq!(eval("`"), Ok(Value::Atom(Atom::Symbol(Sym::NULL))));
    assert_eq!(show("`a"), "`a");
    assert_eq!(show("`"), "`");
    assert_eq!(eval("`a").unwrap().type_code(), -11);
}

#[test]
fn vec_lit_string() {
    assert_eq!(
        eval("\"abc\""),
        Ok(vec_of(Column::Char(vec!['a', 'b', 'c'])))
    );
    assert_eq!(eval("\"\""), Ok(vec_of(Column::Char(vec![]))));
    assert_eq!(show("\"abc\""), "\"abc\"");
    assert_eq!(show("\"\""), "\"\"");
    assert_eq!(eval("\"abc\"").unwrap().type_code(), 10);
    // one character is a char atom
    assert_eq!(eval("\"a\""), Ok(Value::Atom(Atom::Character('a'))));
    assert_eq!(
        eval("\"a\\nb\""),
        Ok(vec_of(Column::Char(vec!['a', '\n', 'b'])))
    );
}

#[test]
fn vec_lit_assign() {
    let mut i = Interpreter::new();
    assert_eq!(i.eval_line("v:1 2 3"), Ok(Some(longs(&[1, 2, 3]))));
    assert_eq!(i.eval_line("v"), Ok(Some(longs(&[1, 2, 3]))));
    i.eval_line("x:y:1 2 3").unwrap();
    assert_eq!(i.get("x"), Some(&longs(&[1, 2, 3])));
    assert_eq!(i.get("y"), Some(&longs(&[1, 2, 3])));
    i.eval_line("name:\"Alice\"").unwrap();
    assert_eq!(
        i.eval_line("name").unwrap().unwrap().to_string(),
        "\"Alice\""
    );
    i.eval_line("s:`a`b").unwrap();
    assert_eq!(i.eval_line("s").unwrap().unwrap(), syms(&["a", "b"]));
}

#[test]
fn vec_lit_parens_and_comments() {
    assert_eq!(eval("(1 2 3)"), Ok(longs(&[1, 2, 3])));
    assert_eq!(eval("((1 2 3))"), Ok(longs(&[1, 2, 3])));
    assert_eq!(eval("1 2 3 / a comment"), Ok(longs(&[1, 2, 3])));
    assert_eq!(eval("1 2 3 /"), Ok(longs(&[1, 2, 3])));
}

#[test]
fn vec_lit_arithmetic_evaluates() {
    let l = |src: &str| eval(src);
    assert_eq!(l("1 2 3+1"), Ok(longs(&[2, 3, 4])));
    assert_eq!(l("1+1 2 3"), Ok(longs(&[2, 3, 4])));
    assert_eq!(l("- 1 2 3"), Ok(longs(&[-1, -2, -3])));
    assert_eq!(l("1 2+3 4"), Ok(longs(&[4, 6])));
    assert_eq!(l("2*1 2"), Ok(longs(&[2, 4])));
    assert_eq!(l("1 2 3 % 2"), Ok(floats(&[0.5, 1.0, 1.5])));
    assert_eq!(l("1 2.5 + 1"), Ok(floats(&[2.0, 3.5])));
    assert_eq!(l("1 0N 3 + 1"), Ok(longs(&[2, N, 4])));
    assert_eq!(l("101b+1"), Ok(longs(&[2, 1, 2])));
    assert_eq!(l("1 2 3+4 5"), Err(QError::Length));
    assert_eq!(l("`a`b + 1"), Err(QError::Type));
    assert_eq!(l("\"abc\" + 1"), Err(QError::Type));
    assert_eq!(l("9223372036854775807 1 + 1"), Err(QError::Overflow));
    // right to left, and the negative-literal trap
    assert_eq!(l("1 2 3 * 2 + 1"), Ok(longs(&[3, 6, 9])));
    assert_eq!(l("2 -1 + 1"), Ok(longs(&[3, 0])));
}

#[test]
fn vec_lit_float_suffix_only_on_the_last_item() {
    for src in [
        "1f 2", "1 2f 3", "1.5f 2", "1f -2", "1f 0N", "1f -.5", "1f .5", "1f\t2",
    ] {
        assert!(
            matches!(eval(src), Err(QError::Parse(m)) if m.starts_with("invalid literal: ")),
            "{src}"
        );
    }
    assert_eq!(eval("1 2 3f"), Ok(floats(&[1.0, 2.0, 3.0])));
    assert_eq!(eval("1 2.5 3f"), Ok(floats(&[1.0, 2.5, 3.0])));
    assert_eq!(eval("2f"), Ok(Value::Atom(Atom::Float(2.0))));
    assert_eq!(eval("1.0 2"), Ok(floats(&[1.0, 2.0])));
    assert_eq!(eval("1f - 2"), Ok(Value::Atom(Atom::Float(-1.0))));
    assert_eq!(eval("1f /c"), Ok(Value::Atom(Atom::Float(1.0))));
}

#[test]
fn vec_lit_mixed_kinds_are_application() {
    // an atom, or a vector indexed by something that is not a long or boolean, is a type error
    for src in [
        "1 \"a\"",
        "1 `a",
        "`a 1",
        "\"a\" \"b\"",
        "`a `b",
        "1 2 \"ab\"",
        "\"a\" 1.5",
        "1 `a`b",
        "1 (2)",
    ] {
        assert_eq!(eval(src), Err(QError::Type), "{src}");
    }
    // an undefined name is reported once the argument has been evaluated
    for src in [
        "x 1 2", "1 2 x", "1 x", "x 1.5", "x 1b", "x `a`b", "x 101b", "x \"ab\"", "x (1)",
    ] {
        assert_eq!(eval(src), Err(QError::Undefined("x".into())), "{src}");
    }
    assert_eq!(eval("x y"), Err(QError::Undefined("y".into())));
    // a vector indexed by a long or boolean vector
    assert_eq!(show("\"ab\" 1"), "\"b\"");
    assert_eq!(show("101b 1"), "0b");
    assert_eq!(show("\"ab\" 0b"), "\"a\"");
    assert_eq!(show("\"ab\" 101b"), "\"bab\"");
}

#[test]
fn vec_lit_application_does_not_swallow_other_errors() {
    assert_eq!(eval("1 +"), Err(QError::parse("unexpected end of input")));
    assert_eq!(
        eval("1 2 3)"),
        Err(QError::parse("unexpected ) after expression"))
    );
    assert_eq!(eval("zz"), Err(QError::Undefined("zz".into())));
}

fn run_text(v: &[i64]) -> String {
    v.iter().map(i64::to_string).collect::<Vec<_>>().join(" ")
}

proptest! {
    #[test]
    fn vec_lit_long_roundtrip(v in proptest::collection::vec((i64::MIN + 1)..=i64::MAX, 2..50)) {
        let got = eval(&run_text(&v)).unwrap();
        prop_assert_eq!(&got, &longs(&v));
        prop_assert_eq!(eval(&got.to_string()).unwrap(), got);
    }

    #[test]
    fn vec_lit_float_roundtrip(v in proptest::collection::vec(-100_000i64..=100_000, 2..50)) {
        let f: Vec<f64> = v.iter().map(|&n| n as f64).collect();
        let src = format!("{}f", run_text(&v));
        let got = eval(&src).unwrap();
        prop_assert_eq!(&got, &floats(&f));
        prop_assert_eq!(eval(&got.to_string()).unwrap(), got);
    }
}

#[test]
fn vec_lit_glued_letters_are_invalid_literals() {
    for src in [
        "1 2 3j", "2x", "1E3 2", "10n", "1x0N", "1j", "0x01", "1F", "1bb", "1foo", "2x:1",
    ] {
        assert!(
            matches!(eval(src), Err(QError::Parse(m)) if m.starts_with("invalid literal: ")),
            "{src}"
        );
    }
    assert_eq!(eval("1 2 3j"), Err(QError::parse("invalid literal: 3j...")));
    // the handled suffixes still lex
    for src in ["1b", "1f", "1e3", "0N", "0n", "0w", "101b", "1.5e-3"] {
        assert!(eval(src).is_ok(), "{src}");
    }
}

#[test]
fn vec_lit_parenthesised_semicolon_is_general_lists_nyi() {
    let nyi = Err(QError::Nyi("general lists".into()));
    for src in ["(1;2;3)", "()", "(1 2;3)", "(1;)", "(;)", "(;1)", "(;;)"] {
        assert_eq!(eval(src), nyi, "{src}");
    }
}

#[test]
fn vec_lit_atom_plus_symbol_is_type() {
    assert_eq!(eval("1+`a"), Err(QError::Type));
    assert_eq!(eval("`a+1"), Err(QError::Type));
}

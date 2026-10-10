//! Application and indexing: `f x`, `v i`, `f[a]`, `v[i]`, builtin names and `neg`.
use oxidedb::language::ast::Verb;
use oxidedb::language::ops;
use oxidedb::types::atom::Atom;
use oxidedb::types::column::Column;
use oxidedb::types::sym::Sym;
use oxidedb::types::value::Value;
use oxidedb::{Interpreter, QError};
use proptest::prelude::*;
use std::rc::Rc;

const N: i64 = i64::MIN;

fn eval(src: &str) -> Result<Value, QError> {
    Interpreter::new()
        .eval_line(src)
        .map(|v| v.expect("a value"))
}

/// Evaluate lines in one session; the last result.
fn session(lines: &[&str]) -> Result<Value, QError> {
    let mut i = Interpreter::new();
    let mut last = None;
    for l in lines {
        last = i.eval_line(l)?;
    }
    Ok(last.expect("a value"))
}

fn show(lines: &[&str]) -> String {
    session(lines)
        .unwrap_or_else(|e| panic!("{lines:?}: {e}"))
        .to_string()
}

fn err(lines: &[&str]) -> QError {
    session(lines).expect_err(&format!("{lines:?}"))
}

fn vec_of(c: Column) -> Value {
    Value::Vector(Rc::new(c))
}

fn longs(v: &[i64]) -> Value {
    vec_of(Column::Long(v.to_vec()))
}

fn long(n: i64) -> Value {
    Value::Atom(Atom::Integer(n))
}

const V: &str = "v:10 20 30";

#[test]
fn apply_til_juxtaposition() {
    assert_eq!(eval("til 5"), Ok(longs(&[0, 1, 2, 3, 4])));
    // juxtaposition takes the whole expression on its right: til (3+2)
    assert_eq!(eval("til 3+2"), Ok(longs(&[0, 1, 2, 3, 4])));
    assert_eq!(eval("til 3 + 2"), Ok(longs(&[0, 1, 2, 3, 4])));
    assert_eq!(eval("til 0"), Ok(longs(&[])));
    assert_eq!(eval("til[3]"), Ok(longs(&[0, 1, 2])));
    assert_eq!(eval("(til 3)"), Ok(longs(&[0, 1, 2])));
    // an application on the right of a verb
    assert_eq!(eval("1 + til 3"), Ok(longs(&[1, 2, 3])));
    assert_eq!(eval("10 * til 3 + 1"), Ok(longs(&[0, 10, 20, 30])));
    assert_eq!(eval("til -1"), Err(QError::Domain));
    assert_eq!(eval("til 0N"), Err(QError::Domain));
    assert_eq!(eval("til 3.0"), Err(QError::Type));
    assert_eq!(eval("til `a"), Err(QError::Type));
}

#[test]
fn apply_count_chain() {
    assert_eq!(eval("count til 5"), Ok(long(5)));
    assert_eq!(eval("count count til 5"), Ok(long(1)));
    assert_eq!(eval("count til count til 3"), Ok(long(3)));
    assert_eq!(eval("count 1 2 3"), Ok(long(3)));
    assert_eq!(eval("count \"abc\""), Ok(long(3)));
    assert_eq!(eval("count `a`b"), Ok(long(2)));
    assert_eq!(eval("count 5"), Ok(long(1)));
    assert_eq!(eval("count[til 4]"), Ok(long(4)));
    assert_eq!(eval("count til 3 + 2"), Ok(long(5)));
    assert_eq!(eval("1 + count 1 2 3"), Ok(long(4)));
}

#[test]
fn apply_index_atom() {
    assert_eq!(show(&[V, "v[1]"]), "20");
    assert_eq!(show(&[V, "v 1"]), "20");
    assert_eq!(show(&[V, "v[0]"]), "10");
    assert_eq!(show(&[V, "v[2]"]), "30");
    assert_eq!(show(&[V, "i:2", "v i"]), "30");
    assert_eq!(show(&[V, "i:2", "v[i]"]), "30");
    assert_eq!(show(&[V, "v 0 + 1"]), "20");
    assert_eq!(show(&[V, "v[0] + 1"]), "11");
    // strings, symbols, booleans and floats index like any vector
    assert_eq!(show(&["\"abc\" 1"]), "\"b\"");
    assert_eq!(show(&["\"abc\"[2]"]), "\"c\"");
    assert_eq!(show(&["`a`b`c 1"]), "`b");
    assert_eq!(show(&["101b 1"]), "0b");
    assert_eq!(show(&["1.5 2.5[1]"]), "2.5");
    // literals and parenthesised vectors take brackets too
    assert_eq!(show(&["1 2 3[1]"]), "2");
    assert_eq!(show(&["(10 20 30)[1]"]), "20");
    assert_eq!(show(&["(10 20 30) 1"]), "20");
    // the index is any long atom, a boolean counts as 0 or 1
    assert_eq!(show(&[V, "v 1b"]), "20");
    assert_eq!(show(&[V, "v[0b]"]), "10");
    assert_eq!(
        eval("(10 20 30) 1"),
        Ok(Value::Atom(Atom::Integer(20))),
        "an atom, not a one-item vector"
    );
}

#[test]
fn apply_index_vector() {
    assert_eq!(session(&[V, "v[0 2]"]), Ok(longs(&[10, 30])));
    assert_eq!(session(&[V, "v 0 2"]), Ok(longs(&[10, 30])));
    assert_eq!(session(&[V, "v[2 1 0]"]), Ok(longs(&[30, 20, 10])));
    assert_eq!(session(&[V, "v[0 0 0 0]"]), Ok(longs(&[10, 10, 10, 10])));
    assert_eq!(session(&[V, "v[til 3]"]), Ok(longs(&[10, 20, 30])));
    assert_eq!(session(&[V, "v til 2"]), Ok(longs(&[10, 20])));
    assert_eq!(session(&[V, "i:0 2", "v i"]), Ok(longs(&[10, 30])));
    assert_eq!(session(&[V, "v[til 0]"]), Ok(longs(&[])));
    // a boolean vector is read as 0 and 1 (not as a mask), as in q
    assert_eq!(session(&[V, "v[101b]"]), Ok(longs(&[20, 10, 20])));
    assert_eq!(show(&["\"abc\" 0 2"]), "\"ac\"");
    assert_eq!(show(&["`a`b`c 2 0"]), "`c`a");
    assert_eq!(show(&["101b 0 1 2"]), "101b");
    assert_eq!(show(&["1.5 2.5 3.5[2 0]"]), "3.5 1.5");
    // the result keeps the type, even when empty
    assert_eq!(
        eval("\"abc\"[til 0]").unwrap().type_code(),
        10,
        "empty string index"
    );
    assert_eq!(eval("`a`b[til 0]").unwrap().type_code(), 11);
    assert_eq!(eval("1.5 2[til 0]").unwrap().type_code(), 9);
    // the result of an index can be indexed again
    assert_eq!(show(&[V, "v[0 2][1]"]), "30");
    assert_eq!(show(&[V, "v[0 2] 1"]), "30");
}

#[test]
fn apply_index_out_of_range_is_null() {
    assert_eq!(session(&[V, "v[5]"]), Ok(long(N)));
    assert_eq!(session(&[V, "v[3]"]), Ok(long(N)));
    assert_ne!(session(&[V, "v[2]"]), Ok(long(N)));
    assert_eq!(session(&[V, "v[-1]"]), Ok(long(N)));
    assert_eq!(session(&[V, "v -1"]), Ok(long(N)));
    assert_eq!(session(&[V, "v 0N"]), Ok(long(N)));
    assert_eq!(session(&[V, "v[9223372036854775807]"]), Ok(long(N)));
    assert_eq!(session(&[V, "v[1 5]"]), Ok(longs(&[20, N])));
    assert_eq!(session(&[V, "v[-1 0 3]"]), Ok(longs(&[N, 10, N])));
    // each type's own null
    assert_eq!(show(&["`a`b[5]"]), "`");
    assert_eq!(eval("`a`b[5]"), Ok(Value::Atom(Atom::Symbol(Sym::NULL))));
    assert_eq!(eval("\"abc\"[5]"), Ok(Value::Atom(Atom::Character(' '))));
    assert_eq!(show(&["\"abc\"[5]"]), "\" \"");
    assert_eq!(eval("101b[7]"), Ok(Value::Atom(Atom::Boolean(false))));
    assert!(matches!(
        eval("1.5 2[5]"),
        Ok(Value::Atom(Atom::Float(f))) if f.is_nan()
    ));
    assert_eq!(show(&["1.5 2[5]"]), "0n");
    assert_eq!(show(&["\"abc\"[1 5]"]), "\"b \"");
    assert_eq!(show(&["`a`b`c[1 5]"]), "`b`");
    assert_eq!(show(&["101b[0 9]"]), "10b");
    assert_eq!(show(&["1.5 2.5[1 9]"]), "2.5 0n");
    // an empty vector has no valid index
    assert_eq!(session(&["e:til 0", "e[0]"]), Ok(long(N)));
    assert_eq!(session(&["e:til 0", "e 0 1"]), Ok(longs(&[N, N])));
    // never an error
    assert_eq!(session(&[V, "v[v]"]), Ok(longs(&[N, N, N])));
}

#[test]
fn apply_index_type_errors() {
    // float, symbol and character indexes
    for src in [
        "v[1.0]",
        "v 1.0",
        "v[1 2f]",
        "v[0.5]",
        "v[`a]",
        "v[`a`b]",
        "v[\"a\"]",
        "v[\"ab\"]",
        "v 0n",
    ] {
        assert_eq!(err(&[V, src]), QError::Type, "{src}");
    }
    // an atom is not indexable (q treats an integer atom applied to an argument as a
    // file handle; O does not imitate that)
    assert_eq!(err(&["x:5", "x 1"]), QError::Type);
    assert_eq!(err(&["x:5", "x[1]"]), QError::Type);
    assert_eq!(err(&["x:5", "x[]"]), QError::Type);
    assert_eq!(err(&["(5) 0"]), QError::Type);
    assert_eq!(err(&["x:5", "x -1"]), QError::Type);
    assert_eq!(err(&["x:`a", "x 0"]), QError::Type);
    assert_eq!(eval("1 \"a\""), Err(QError::Type));
    assert_eq!(eval("`a `b"), Err(QError::Type));
    assert_eq!(eval("1b 0b"), Err(QError::Type));
    // a vector index must be long or boolean throughout
    assert_eq!(err(&[V, "v[1.5 2.5]"]), QError::Type);
}

#[test]
fn apply_index_too_many_is_rank() {
    assert_eq!(err(&[V, "v[0;1]"]), QError::Rank);
    assert_eq!(err(&[V, "v[0;1;2]"]), QError::Rank);
    assert_eq!(err(&["\"abc\"[0;0]"]), QError::Rank);
    assert_eq!(eval("til[1;2]"), Err(QError::Rank));
    assert_eq!(eval("count[1;2]"), Err(QError::Rank));
    assert_eq!(eval("neg[1;2]"), Err(QError::Rank));
    // O: an empty bracket on a function is a rank error (q passes a null to it)
    assert_eq!(eval("til[]"), Err(QError::Rank));
    assert_eq!(eval("count[]"), Err(QError::Rank));
    // an empty bracket on a vector gives the vector back
    assert_eq!(session(&[V, "v[]"]), Ok(longs(&[10, 20, 30])));
    assert_eq!(show(&["\"abc\"[]"]), "\"abc\"");
}

#[test]
fn apply_dyadic_verbs_wired() {
    assert_eq!(show(&["1 2 3 = 1 5 3"]), "101b");
    assert_eq!(show(&["(til 3) = 0 2 2"]), "101b");
    assert_eq!(show(&[V, "v[0 2] = 10 30"]), "11b");
    assert_eq!(show(&[V, "v[0 2] > 15"]), "01b");
    // right to left: v (1 > 10) indexes with 0b
    assert_eq!(show(&[V, "v 1 > 10"]), "10");
    // Deliberate deviation: q accepts a boolean in til (`til 1b` is `,0`); O is a type error.
    assert_eq!(eval("count til 4 > 2"), Err(QError::Type));
    // # , ! reach their kernels with these operands
    let l = longs(&[0, 1, 2]);
    for (src, verb) in [
        ("2 # til 3", Verb::Take),
        ("2 , til 3", Verb::Join),
        ("2 ! til 3", Verb::Key),
    ] {
        assert_eq!(eval(src), ops::dyad(verb, &long(2), &l), "{src}");
    }
}

#[test]
fn apply_rhs_first() {
    assert_eq!(show(&["x:1", "x+x:2"]), "4");
    // the argument is evaluated before the vector it indexes
    assert_eq!(show(&[V, "v[i:1]"]), "20");
    assert_eq!(show(&[V, "v[i:1]", "i"]), "1");
    assert_eq!(show(&[V, "v i:2"]), "30");
    // (a:1) (a:2): the argument assigns a:2 first, then the function side a:1
    let mut i = Interpreter::new();
    assert!(i.eval_line("(a:1) (a:2)").is_err());
    assert_eq!(i.get("a"), Some(&long(1)));
    // bracket arguments run right to left
    assert_eq!(err(&["til[x:1;x]"]), QError::Undefined("x".into()));
    assert_eq!(err(&["til[x;x:1]"]), QError::Rank);
    assert_eq!(err(&["til[x:1;y:2]", "x"]), QError::Rank);
    let mut i = Interpreter::new();
    assert!(i.eval_line("til[x:1;y:2]").is_err());
    assert_eq!((i.get("x"), i.get("y")), (Some(&long(1)), Some(&long(2))));
    // an undefined function does not hide the argument's side effect
    let mut i = Interpreter::new();
    assert_eq!(i.eval_line("zz a:3"), Err(QError::Undefined("zz".into())));
    assert_eq!(i.get("a"), Some(&long(3)));
}

#[test]
fn apply_names_variables_before_builtins() {
    // a variable (set through the library) shadows the builtin of the same name
    let mut i = Interpreter::new();
    i.set("count", longs(&[10, 20, 30]));
    assert_eq!(i.eval_line("count 1").unwrap(), Some(long(20)));
    assert_eq!(i.eval_line("count[2]").unwrap(), Some(long(30)));
    let mut i = Interpreter::new();
    assert_eq!(i.eval_line("count 1 2 3").unwrap(), Some(long(3)));
    // an unknown name is undefined
    assert_eq!(eval("foo 1"), Err(QError::Undefined("foo".into())));
    assert_eq!(eval("foo[1]"), Err(QError::Undefined("foo".into())));
    assert_eq!(eval("Til 3"), Err(QError::Undefined("Til".into())));
}

#[test]
fn apply_builtin_names_cannot_be_assigned() {
    for name in ["til", "count", "neg"] {
        let mut i = Interpreter::new();
        assert_eq!(
            i.eval_line(&format!("{name}:5")),
            Err(QError::parse(format!("cannot assign to builtin {name}"))),
            "{name}"
        );
        // and it still works as the builtin afterwards
        assert!(i.eval_line(&format!("{name} 3")).is_ok());
    }
    // refused while parsing, so nothing on the right runs, in any position
    let mut i = Interpreter::new();
    assert!(i.eval_line("til:a:7").is_err());
    assert_eq!(i.get("a"), None);
    assert_eq!(
        i.eval_line("1+count:2"),
        Err(QError::parse("cannot assign to builtin count"))
    );
    // other names are free
    assert_eq!(eval("tilde:5"), Ok(long(5)));
    assert_eq!(eval("counts:5"), Ok(long(5)));
}

#[test]
fn apply_arguments_run_before_the_name_is_resolved() {
    // `w` is unknown until the argument assigns it: the argument is evaluated first, so the
    // name then resolves to the new variable and indexes it (q gives the same).
    assert_eq!(eval("w[w:0 1]"), Ok(longs(&[0, 1])));
    // the same for a builtin name: a variable assigned in the argument is not consulted
    // before the argument has run, and the builtin still applies when no variable exists
    assert_eq!(eval("count[w:0 1]"), Ok(long(2)));
}

#[test]
fn apply_bracket_arguments_report_nyi_tokens() {
    assert_eq!(err(&[V, "v[1 $ 2]"]), QError::Nyi("$".into()));
    assert_eq!(err(&[V, "v[1 {"]), QError::Nyi("{".into()));
    assert_eq!(err(&[V, "v[1 @ 2]"]), QError::Nyi("@".into()));
}

#[test]
fn apply_bare_builtin_is_nyi() {
    for name in ["til", "count", "neg"] {
        assert_eq!(eval(name), Err(QError::Nyi(format!("{name} as a value"))));
    }
    // q prints the builtin; O does not have function values yet
    assert_eq!(eval("f:til"), Err(QError::Nyi("til as a value".into())));
    // but a variable of that name is a plain value
    let mut i = Interpreter::new();
    i.set("til", long(3));
    assert_eq!(i.eval_line("til").unwrap(), Some(long(3)));
}

#[test]
fn apply_neg() {
    assert_eq!(eval("neg 1b"), Ok(long(-1)));
    assert_eq!(eval("neg 0b"), Ok(long(0)));
    assert_eq!(eval("neg 5"), Ok(long(-5)));
    assert_eq!(eval("neg 1 2 3"), Ok(longs(&[-1, -2, -3])));
    assert_eq!(eval("neg 101b"), Ok(longs(&[-1, 0, -1])));
    assert_eq!(show(&["neg 1.5"]), "-1.5");
    assert_eq!(eval("neg 0N"), Ok(long(N)));
    assert_eq!(eval("neg til 3"), Ok(longs(&[0, -1, -2])));
    assert_eq!(eval("neg[5]"), Ok(long(-5)));
    assert_eq!(eval("1 + neg 2"), Ok(long(-1)));
    assert_eq!(eval("neg neg 2"), Ok(long(2)));
    assert_eq!(eval("neg 1 + 2"), Ok(long(-3)));
    assert_eq!(eval("neg `a"), Err(QError::Type));
    assert_eq!(eval("neg[]"), Err(QError::Rank));
    // O's leading minus does the same thing
    assert_eq!(eval("- 1 2 3"), eval("neg 1 2 3"));
}

#[test]
fn apply_general_list_indexes_by_item() {
    let list = Value::List(Rc::new(vec![
        long(1),
        longs(&[2, 3]),
        Value::Atom(Atom::Symbol(Sym::intern("a"))),
    ]));
    let mut i = Interpreter::new();
    i.set("L", list.clone());
    assert_eq!(i.eval_line("L[0]").unwrap(), Some(long(1)));
    assert_eq!(i.eval_line("L 1").unwrap(), Some(longs(&[2, 3])));
    assert_eq!(i.eval_line("L[2]").unwrap().unwrap().to_string(), "`a");
    // out of range gives a null (O: always the long null; q uses the first item's type)
    assert_eq!(i.eval_line("L[3]").unwrap(), Some(long(N)));
    assert_eq!(i.eval_line("L[-1]").unwrap(), Some(long(N)));
    // a vector index gives a list again
    assert_eq!(
        i.eval_line("L[0 1]").unwrap(),
        Some(Value::List(Rc::new(vec![long(1), longs(&[2, 3])])))
    );
    assert_eq!(
        i.eval_line("L[til 0]").unwrap(),
        Some(Value::List(Rc::new(vec![])))
    );
    assert_eq!(i.eval_line("L[1][0]").unwrap(), Some(long(2)));
    assert_eq!(i.eval_line("L[]").unwrap(), Some(list));
    assert_eq!(i.eval_line("L[1.0]"), Err(QError::Type));
    // depth indexing (several indexes at once) is not built yet
    assert_eq!(
        i.eval_line("L[0;0]"),
        Err(QError::Nyi("depth indexing".into()))
    );
    // a general list used as the index: item-wise
    i.set("I", Value::List(Rc::new(vec![long(0), longs(&[1, 0])])));
    i.set("v", longs(&[10, 20, 30]));
    assert_eq!(
        i.eval_line("v I").unwrap(),
        Some(Value::List(Rc::new(vec![long(10), longs(&[20, 10])])))
    );
}

#[test]
fn apply_brackets_bind_tighter_and_chain() {
    assert_eq!(show(&[V, "v[0 2][1]"]), "30");
    assert_eq!(show(&[V, "v[0 2] [1]"]), "30");
    assert_eq!(show(&[V, "v [1]"]), "20");
    assert_eq!(show(&[V, "(v)[1]"]), "20");
    assert_eq!(show(&["w:5 6 7 8", "w[2 3][0]"]), "7");
    assert_eq!(show(&[V, "v[v[0]-9]"]), "20");
    assert_eq!(show(&[V, "v[1] + v[2]"]), "50");
    // a bracket binds to its term: count (til[3] + 1)
    assert_eq!(eval("count til[3] + 1"), Ok(long(3)));
    assert_eq!(eval("count[1 2 3][0]"), Err(QError::Type));
    // application of a bracket result
    assert_eq!(show(&[V, "w:1 0", "v w[0]"]), "20");
    // juxtaposition after a bracket applies the result: v[0 1] 1 is 20
    assert_eq!(show(&[V, "v[0 1] 1"]), "20");
}

#[test]
fn apply_bracket_syntax_errors() {
    assert_eq!(
        err(&[V, "v[0"]),
        QError::parse("expected ']' after arguments")
    );
    assert_eq!(
        err(&[V, "v[0)"]),
        QError::parse("expected ']' after arguments")
    );
    assert_eq!(err(&[V, "v["]), QError::parse("unexpected end of input"));
    assert_eq!(
        err(&[V, "v[0]]"]),
        QError::parse("unexpected ] after expression")
    );
    assert_eq!(eval("[1]"), Err(QError::parse("unexpected [")));
    assert_eq!(eval("1 + [1]"), Err(QError::parse("unexpected [")));
    // elided arguments are projections, not built yet
    assert_eq!(err(&[V, "v[0;]"]), QError::Nyi("elided argument".into()));
    assert_eq!(err(&[V, "v[;1]"]), QError::Nyi("elided argument".into()));
    assert_eq!(err(&[V, "v[;]"]), QError::Nyi("elided argument".into()));
    // item assignment of a single index on a name is built (tests/index_assign.rs); deeper targets are not
    assert_eq!(
        err(&[V, "v[0][0]:5"]),
        QError::Nyi("depth assignment".into())
    );
    // lambdas, adverbs and the other application forms keep their nyi
    assert_eq!(eval("{x} 1"), Err(QError::Nyi("{".into())));
    assert_eq!(eval("1 @ 2"), Err(QError::Nyi("@".into())));
    assert_eq!(eval("1 $ 2"), Err(QError::Nyi("$".into())));
    assert_eq!(eval("til 3/2"), Err(QError::Nyi("adverb '/'".into())));
}

#[test]
fn apply_the_negative_literal_trap() {
    // x -1 is x applied to -1, which is out of range, so a null
    assert_eq!(show(&["x:1 2 3", "x -1"]), "0N");
    assert_eq!(show(&["x:1 2 3", "x - 1"]), "0 1 2");
    assert_eq!(show(&["x:1 2 3", "x-1"]), "0 1 2");
    assert_eq!(show(&["x:1 2 3", "x -1 + 1"]), "1");
    assert_eq!(show(&["x:1 2 3", "x - 1 + 1"]), "-1 0 1");
    // a parenthesised name is still an assignment target, as in q
    assert_eq!(show(&["(x):3", "x"]), "3");
    assert_eq!(show(&["(x):1 2", "x 1"]), "2");
}

#[test]
fn apply_old_nyi_placeholders_are_gone() {
    for src in ["1 \"a\"", "1 `a", "\"ab\" 1", "`a `b", "1b 0b"] {
        assert_ne!(eval(src), Err(QError::Nyi("application".into())), "{src}");
    }
    assert_eq!(eval("1 x"), Err(QError::Undefined("x".into())));
}

fn deep<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .expect("thread panicked")
}

fn shape(src: String) -> Result<String, String> {
    deep(move || eval(&src).map(|v| v.to_string()).map_err(|e| e.to_string()))
}

#[test]
fn apply_chain_limits_hold() {
    let n = 200_000;
    let too_long =
        |r: Result<String, String>| r.is_err_and(|e| e.starts_with("'parse: expression too long"));
    assert!(too_long(shape(format!("{}1", "count ".repeat(n)))));
    assert!(too_long(shape(format!("{}1", "neg ".repeat(n)))));
    assert!(too_long(v_session(format!("{}1", "v ".repeat(n)))));
    // The line spends one, each join one: 1,999 joins are fine, 2,000 are not.
    assert_eq!(shape(format!("{}1", "count ".repeat(1999))), Ok("1".into()));
    assert!(too_long(shape(format!("{}1", "count ".repeat(2000)))));
    // verbs and applications share one budget
    assert!(too_long(shape(format!("{}1", "count 1+".repeat(1000)))));
    // each postfix bracket spends a link and its argument expression: 1 + 2k <= 2000
    let chain = |k: usize| format!("v{}", "[0]".repeat(k));
    assert!(too_long(v_session(chain(5000))));
    assert_eq!(v_session(chain(999)), Err("'type".to_string()));
    assert!(too_long(v_session(chain(1000))));
    // bracket nesting counts as nesting: 127 deep is fine, 128 is not
    let nested = |k: usize| format!("{}0{}", "v[".repeat(k), "]".repeat(k));
    assert_eq!(v_session(nested(127)), Ok("0".to_string()));
    assert_eq!(
        v_session(nested(128)),
        Err("'parse: expression nested too deeply".to_string())
    );
}

/// `v:0 0`, then one expression, on a big stack.
fn v_session(src: String) -> Result<String, String> {
    deep(move || {
        let mut i = Interpreter::new();
        i.eval_line("v:0 0").unwrap();
        i.eval_line(&src)
            .map(|v| v.unwrap().to_string())
            .map_err(|e| e.to_string())
    })
}

proptest! {
    #[test]
    fn apply_count_til_is_n(n in 0i64..5000) {
        prop_assert_eq!(eval(&format!("count til {n}")), Ok(long(n)));
    }

    #[test]
    fn apply_til_index_is_identity_or_null(n in 0i64..200, i in -5i64..260) {
        let want = if (0..n).contains(&i) { i } else { N };
        let src = if i < 0 { format!("(til {n}) ({i})") } else { format!("(til {n}) {i}") };
        prop_assert_eq!(eval(&src), Ok(long(want)), "{}", src);
        prop_assert_eq!(eval(&format!("(til {n})[{i}]")), Ok(long(want)));
    }

    #[test]
    fn apply_index_by_til_count_is_identity(v in proptest::collection::vec(-1000i64..1000, 2..40)) {
        let text = v.iter().map(i64::to_string).collect::<Vec<_>>().join(" ");
        let mut i = Interpreter::new();
        i.eval_line(&format!("v:{text}")).unwrap();
        let got = i.eval_line("v[til count v]").unwrap().unwrap();
        prop_assert_eq!(got, longs(&v));
    }

    #[test]
    fn apply_gather_matches_slice_get(
        v in proptest::collection::vec(-100i64..100, 2..20),
        idx in proptest::collection::vec(-3i64..25, 0..20),
    ) {
        let text = |v: &[i64]| v.iter().map(i64::to_string).collect::<Vec<_>>().join(" ");
        let mut i = Interpreter::new();
        i.eval_line(&format!("v:{}", text(&v))).unwrap();
        let want: Vec<i64> = idx.iter()
            .map(|&k| usize::try_from(k).ok().and_then(|k| v.get(k)).copied().unwrap_or(N))
            .collect();
        // a literal index needs two or more items
        if idx.len() >= 2 {
            let got = i.eval_line(&format!("v {}", text(&idx))).unwrap().unwrap();
            prop_assert_eq!(got, longs(&want));
        }
    }
}

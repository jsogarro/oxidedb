//! Index assignment: `v[i]:x`. Every behaviour below was checked against q 4.1.
use oxidedb::types::atom::Atom;
use oxidedb::types::column::Column;
use oxidedb::types::sym::Sym;
use oxidedb::types::value::Value;
use oxidedb::{Interpreter, QError};
use proptest::prelude::*;
use std::rc::Rc;

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

fn sym(s: &str) -> Value {
    Value::Atom(Atom::Symbol(Sym::intern(s)))
}

#[test]
fn iassign_single() {
    // the statement's value is the assigned item
    assert_eq!(session(&["v:1 2 3", "v[0]:5"]), Ok(long(5)));
    assert_eq!(session(&["v:1 2 3", "v[0]:5", "v"]), Ok(longs(&[5, 2, 3])));
    assert_eq!(session(&["v:1 2 3", "v[1]:5", "v"]), Ok(longs(&[1, 5, 3])));
    assert_eq!(session(&["v:1 2 3", "v[2]:5", "v"]), Ok(longs(&[1, 2, 5])));
    // a boolean atom index counts as 0 or 1, and the index may be a variable
    assert_eq!(show(&["v:1 2 3", "v[1b]:7", "v"]), "1 7 3");
    assert_eq!(show(&["v:1 2 3", "i:2", "v[i]:7", "v"]), "1 2 7");
    // repeated assignment, and an assigned value taken from another item
    assert_eq!(show(&["v:1 2 3", "v[0]:5", "v[0]:6", "v"]), "6 2 3");
    assert_eq!(show(&["v:1 2 3", "v[0]:v[1]", "v"]), "2 2 3");
    // the type is kept, a null is a legal value of its own type
    assert_eq!(show(&["v:1 2 3", "v[0]:0N", "v"]), "0N 2 3");
    assert_eq!(show(&["v:1 2 3.", "v[0]:0n", "v"]), "0n 2 3");
    assert_eq!(show(&["v:1 2 3.", "v[0]:4.", "v"]), "4 2 3f");
    assert_eq!(show(&["v:`a`b", "v[1]:`", "v"]), "`a`");
    assert_eq!(show(&["v:011b", "v[0]:1b", "v"]), "111b");
    assert_eq!(show(&["v:`a`b", "v[0]:`c", "v"]), "`c`b");
}

#[test]
fn iassign_strings() {
    assert_eq!(show(&["s:\"abc\"", "s[0]:\"x\"", "s"]), "\"xbc\"");
    assert_eq!(show(&["s:\"abc\"", "s[0 1]:\"xy\"", "s"]), "\"xyc\"");
    assert_eq!(show(&["s:\"abc\"", "s[0 1]:\"x\"", "s"]), "\"xxc\"");
    assert_eq!(err(&["s:\"abc\"", "s[0]:`x"]), QError::Type);
    assert_eq!(err(&["s:\"abc\"", "s[0]:65"]), QError::Type);
}

#[test]
fn iassign_vector_index() {
    // pairwise
    assert_eq!(show(&["v:1 2 3", "v[0 2]:7 8", "v"]), "7 2 8");
    assert_eq!(show(&["v:1 2 3", "v[2 0]:7 8", "v"]), "8 2 7");
    // an atom value is broadcast
    assert_eq!(show(&["v:1 2 3", "v[0 2]:9", "v"]), "9 2 9");
    assert_eq!(show(&["v:1 2 3", "v[0 1]:5", "v"]), "5 5 3");
    // a one-item index takes a one-item value
    assert_eq!(show(&["v:1 2 3", "v[1 1]:1 2", "v"]), "1 2 3");
    assert_eq!(show(&["v:1 2 3", "v[til 3]:0", "v"]), "0 0 0");
    // a duplicate index: the last one wins
    assert_eq!(show(&["v:1 2 3", "v[0 0]:1 2", "v"]), "2 2 3");
    assert_eq!(show(&["v:1 2 3", "v[0 1 0]:7 8 9", "v"]), "9 8 3");
    // a boolean vector is read as 0 and 1, not as a mask
    assert_eq!(show(&["v:1 2 3", "v[01b]:7 8", "v"]), "7 8 3");
    assert_eq!(show(&["v:1 2 3", "v[101b]:7 8 9", "v"]), "8 9 3");
    // an empty index changes nothing
    assert_eq!(show(&["v:1 2 3", "v[0#0]:5", "v"]), "1 2 3");
    assert_eq!(show(&["v:1 2 3", "v[0#0]:0#0", "v"]), "1 2 3");
    // every column type
    assert_eq!(show(&["v:1 2 3.", "v[0 2]:7 8.", "v"]), "7 2 8f");
    assert_eq!(show(&["v:`a`b`c", "v[0 2]:`x`y", "v"]), "`x`b`y");
    assert_eq!(show(&["v:101b", "v[0 1]:01b", "v"]), "011b");
    assert_eq!(show(&["v:\"abc\"", "v[0 2]:\"xy\"", "v"]), "\"xby\"");
}

#[test]
fn iassign_vector_index_length_mismatch() {
    assert_eq!(err(&["v:1 2 3", "v[0 2]:7 8 9"]), QError::Length);
    assert_eq!(err(&["v:1 2 3", "v[0 1]:1 2 3"]), QError::Length);
    assert_eq!(err(&["v:1 2 3", "v[0 1]:1#4"]), QError::Length);
    assert_eq!(err(&["v:1 2 3", "v[0#0]:5 6"]), QError::Length);
    // the count is checked before the type
    assert_eq!(err(&["v:1 2 3", "v[0 1]:1.5 2 3"]), QError::Length);
    // an atom index takes an atom: a vector value is a type error
    assert_eq!(err(&["v:1 2 3", "v[0]:7 8"]), QError::Type);
    // and the failed assignment changed nothing
    let mut i = Interpreter::new();
    i.eval_line("v:1 2 3").unwrap();
    assert!(i.eval_line("v[0 2]:7 8 9").is_err());
    assert_eq!(i.get("v"), Some(&longs(&[1, 2, 3])));
}

#[test]
fn iassign_out_of_range_is_length() {
    // q has no 'index error: an out-of-range amend is 'length (a read gives a null)
    assert_eq!(err(&["v:1 2 3", "v[3]:9"]), QError::Length);
    assert_eq!(err(&["v:1 2 3", "v[5]:9"]), QError::Length);
    assert_eq!(err(&["v:1 2 3", "v[-1]:7"]), QError::Length);
    assert_eq!(err(&["v:1 2 3", "v[0N]:7"]), QError::Length);
    assert_eq!(err(&["v:1 2 3", "v[0 5]:7 8"]), QError::Length);
    assert_eq!(err(&["v:1 2 3", "v[0 -1]:7"]), QError::Length);
    assert_eq!(err(&["v:1 2 3", "v[0 0N]:5 6"]), QError::Length);
    assert_eq!(err(&["v:0#1", "v[0]:1"]), QError::Length);
    assert_eq!(err(&["s:\"abc\"", "s[3]:\"x\""]), QError::Length);
    // the range is checked before the value's type and shape
    assert_eq!(err(&["v:1 2 3", "v[5]:1.5"]), QError::Length);
    assert_eq!(err(&["v:1 2 3", "v[9]:7 8"]), QError::Length);
    assert_eq!(err(&["v:1 2 3", "v[1 5]:1.5 2"]), QError::Length);
    // nothing is assigned when one index is out of range: no partial update
    assert_eq!(err(&["v:1 2 3", "v[0 5]:7 8"]), QError::Length);
    let mut i = Interpreter::new();
    i.eval_line("v:1 2 3").unwrap();
    assert!(i.eval_line("v[0 5]:7 8").is_err());
    assert_eq!(i.get("v"), Some(&longs(&[1, 2, 3])));
    // the contrast: a read is a null
    assert_eq!(show(&["v:1 2 3", "v[5]"]), "0N");
}

#[test]
fn iassign_index_type() {
    assert_eq!(err(&["v:1 2 3", "v[1.]:5"]), QError::Type);
    assert_eq!(err(&["v:1 2 3", "v[`a]:5"]), QError::Type);
    assert_eq!(err(&["v:1 2 3", "v[1.5]:9"]), QError::Type);
    assert_eq!(err(&["v:1 2 3", "v[`a`b]:5"]), QError::Type);
    // a character index is a type error here (q reads it as its code point)
    assert_eq!(err(&["v:1 2 3", "v[\"a\"]:9"]), QError::Type);
}

#[test]
fn iassign_type_mismatch_is_type() {
    // no promotion in either direction, no bool/long/char mixing
    for (setup, value) in [
        ("v:1 2 3", "1.5"),
        ("v:1 2 3", "`a"),
        ("v:1 2 3", "1b"),
        ("v:1 2 3", "\"a\""),
        ("v:1 2 3", "0n"),
        ("v:1 2 3.", "5"),
        ("v:1 2 3.", "0N"),
        ("v:1 2 3.", "1b"),
        ("v:1 2 3.", "`a"),
        ("v:011b", "5"),
        ("v:011b", "1.5"),
        ("v:`a`b", "\"c\""),
        ("v:`a`b", "1"),
        ("v:\"abc\"", "`x"),
        ("v:\"abc\"", "65"),
    ] {
        let line = format!("v[0]:{value}");
        assert_eq!(err(&[setup, &line]), QError::Type, "{setup}; {line}");
        let line = format!("v[0 1]:{value}");
        assert_eq!(err(&[setup, &line]), QError::Type, "{setup}; {line}");
    }
    // a vector of the wrong type, and one with a wrong item
    assert_eq!(err(&["v:1 2 3", "v[0 1]:4 5."]), QError::Type);
    assert_eq!(err(&["v:1 2 3", "v[0 1]:`a`b"]), QError::Type);
    assert_eq!(err(&["v:1 2 3.", "v[0 1]:4 5"]), QError::Type);
    // an empty index still checks the type
    assert_eq!(err(&["v:1 2 3", "v[0#0]:1.5"]), QError::Type);
    // a failed assignment leaves the variable as it was
    let mut i = Interpreter::new();
    i.eval_line("v:1 2 3").unwrap();
    assert!(i.eval_line("v[0 1]:7 8.").is_err());
    assert_eq!(i.get("v"), Some(&longs(&[1, 2, 3])));
}

#[test]
fn iassign_general_list_any_type() {
    let mixed = Value::List(Rc::new(vec![long(1), sym("a"), longs(&[7, 8])]));
    let run = |line: &str| {
        let mut i = Interpreter::new();
        i.set("l", mixed.clone());
        i.set("w", Value::List(Rc::new(vec![long(2), sym("b")])));
        let v = i.eval_line(line).map(|v| v.unwrap());
        (v, i.get("l").cloned().unwrap())
    };
    // any type goes into any slot, and a vector stays one item
    let (v, l) = run("l[0]:\"x\"");
    assert_eq!(v, Ok(Value::Atom(Atom::Character('x'))));
    assert_eq!(
        l,
        Value::List(Rc::new(vec![
            Value::Atom(Atom::Character('x')),
            sym("a"),
            longs(&[7, 8])
        ]))
    );
    let (_, l) = run("l[2]:9");
    assert_eq!(l, Value::List(Rc::new(vec![long(1), sym("a"), long(9)])));
    let (_, l) = run("l[0]:1 2 3");
    assert_eq!(
        l,
        Value::List(Rc::new(vec![longs(&[1, 2, 3]), sym("a"), longs(&[7, 8])]))
    );
    // pairwise from a list value, broadcast of an atom
    let (v, l) = run("l[0 1]:w");
    assert_eq!(v, Ok(Value::List(Rc::new(vec![long(2), sym("b")]))));
    assert_eq!(
        l,
        Value::List(Rc::new(vec![long(2), sym("b"), longs(&[7, 8])]))
    );
    let (v, l) = run("l[0 1]:7");
    assert_eq!(v, Ok(longs(&[7, 7])));
    assert_eq!(
        l,
        Value::List(Rc::new(vec![long(7), long(7), longs(&[7, 8])]))
    );
    // length and range
    assert_eq!(run("l[0 1 2]:w").0, Err(QError::Length));
    assert_eq!(run("l[3]:1").0, Err(QError::Length));
    assert_eq!(run("l[-1]:1").0, Err(QError::Length));
    // no partial update on error
    assert_eq!(run("l[0 3]:w").1, mixed);
    // a general list that becomes homogeneous collapses to a simple list, as in q
    let pair = Value::List(Rc::new(vec![long(1), sym("a")]));
    let mut i = Interpreter::new();
    i.set("l", pair.clone());
    i.eval_line("l[1]:3").unwrap();
    assert_eq!(i.get("l"), Some(&longs(&[1, 3])));
    assert_eq!(i.eval_line("l").unwrap().unwrap().type_code(), 7);
    i.set("l", pair.clone());
    i.eval_line("l[0 1]:7").unwrap();
    assert_eq!(i.get("l"), Some(&longs(&[7, 7])));
    // a list of vectors does not collapse
    let nested = Value::List(Rc::new(vec![longs(&[1, 2]), longs(&[3, 4])]));
    i.set("l", nested);
    i.eval_line("l[0]:5").unwrap();
    assert_eq!(i.get("l").unwrap().type_code(), 0);
    // a list value goes into a simple vector only if its items are atoms of the right type
    let ok = Value::List(Rc::new(vec![long(8), long(9)]));
    i.set("v", longs(&[1, 2, 3]));
    i.set("w", ok);
    i.eval_line("v[0 1]:w").unwrap();
    assert_eq!(i.get("v"), Some(&longs(&[8, 9, 3])));
    i.set("w", Value::List(Rc::new(vec![long(8), sym("a")])));
    assert_eq!(i.eval_line("v[0 1]:w"), Err(QError::Type));
    i.set("w", Value::List(Rc::new(vec![long(8), longs(&[1, 2])])));
    assert_eq!(i.eval_line("v[0 1]:w"), Err(QError::Type));
    assert_eq!(i.get("v"), Some(&longs(&[8, 9, 3])));
}

#[test]
fn iassign_cow_copies() {
    // another name sharing the vector must not change
    assert_eq!(show(&["v:1 2 3", "w:v", "v[0]:0", "w"]), "1 2 3");
    assert_eq!(show(&["v:1 2 3", "w:v", "v[0]:0", "v"]), "0 2 3");
    assert_eq!(show(&["v:1 2 3", "w:v", "w[0]:0", "v"]), "1 2 3");
    assert_eq!(show(&["v:1 2 3", "w:v", "v[0 1]:9", "w"]), "1 2 3");
    assert_eq!(show(&["a:b:c:1 2 3", "b[1]:0", "a"]), "1 2 3");
    assert_eq!(show(&["a:b:c:1 2 3", "b[1]:0", "c"]), "1 2 3");
    // the value assigned is not changed by later assignments to the target
    assert_eq!(
        show(&["v:1 2 3", "u:7 8", "v[0 1]:u", "v[0]:0", "u"]),
        "7 8"
    );
    // a vector held by the host is not changed either
    let orig = longs(&[1, 2, 3]);
    let mut i = Interpreter::new();
    i.set("v", orig.clone());
    i.eval_line("v[0]:9").unwrap();
    assert_eq!(orig, longs(&[1, 2, 3]));
    assert_eq!(i.get("v"), Some(&longs(&[9, 2, 3])));
    // an unshared vector is changed in place: no copy
    let Some(Value::Vector(before)) = i.get("v").cloned() else {
        panic!()
    };
    drop(before);
    let p0 = match i.get("v") {
        Some(Value::Vector(c)) => match &**c {
            Column::Long(v) => v.as_ptr(),
            _ => panic!(),
        },
        _ => panic!(),
    };
    i.eval_line("v[1]:8").unwrap();
    let p1 = match i.get("v") {
        Some(Value::Vector(c)) => match &**c {
            Column::Long(v) => v.as_ptr(),
            _ => panic!(),
        },
        _ => panic!(),
    };
    assert_eq!(p0, p1, "an unshared vector is amended in place");
    // general lists copy too
    let l = Value::List(Rc::new(vec![long(1), sym("a")]));
    i.set("l", l.clone());
    i.eval_line("m:l").unwrap();
    i.eval_line("l[0]:\"x\"").unwrap();
    assert_eq!(i.get("m"), Some(&l));
    assert_ne!(i.get("l"), Some(&l));
}

#[test]
fn iassign_undefined_name() {
    // q treats an undefined name as an empty list: every index is out of range
    assert_eq!(err(&["u[0]:5"]), QError::Length);
    assert_eq!(err(&["u[0 1]:5"]), QError::Length);
    // the value is evaluated first, so its own error wins
    assert_eq!(err(&["u[0]:nosuch"]), QError::Undefined("nosuch".into()));
    assert_eq!(
        err(&["v:1 2 3", "v[nosuch]:1"]),
        QError::Undefined("nosuch".into())
    );
    // and nothing got defined
    let mut i = Interpreter::new();
    assert!(i.eval_line("u[0]:5").is_err());
    assert!(i.get("u").is_none());
}

#[test]
fn iassign_target_must_be_a_list() {
    // an atom cannot be amended
    assert_eq!(err(&["x:5", "x[0]:1"]), QError::Type);
    assert_eq!(err(&["x:`a", "x[0]:`b"]), QError::Type);
    // a builtin name is not assignable, as for `count:1`
    assert_eq!(
        err(&["count[0]:1"]),
        QError::parse("cannot assign to builtin count")
    );
    // anything but a plain name is not built (depth assignment)
    let depth = QError::Nyi("depth assignment".into());
    assert_eq!(err(&["v:1 2 3", "v[0][0]:9"]), depth);
    assert_eq!(err(&["v:1 2 3", "v[0;1]:9"]), depth);
    assert_eq!(err(&["v:1 2 3", "v[]:9"]), depth);
    assert_eq!(err(&["1 2 3[0]:9"]), depth);
    assert_eq!(err(&["5[0]:1"]), depth);
    assert_eq!(err(&["til[3][0]:1"]), depth);
    // parentheses around the target or the whole index expression change nothing
    assert_eq!(show(&["v:1 2 3", "(v)[0]:5", "v"]), "5 2 3");
    assert_eq!(show(&["v:1 2 3", "(v[0]):5", "v"]), "5 2 3");
    assert_eq!(show(&["v:1 2 3", "((v[0])):5", "v"]), "5 2 3");
    // compound assignment is not built
    let compound = QError::Nyi("compound assignment".into());
    assert_eq!(err(&["v:1 2 3", "v[0]+:1"]), compound);
    assert_eq!(err(&["v:1 2 3", "v[0 1]-:1 1"]), compound);
    assert_eq!(err(&["x:1", "x+:1"]), compound);
    // a variable of the same name as nothing built-in works with juxtaposition left alone
    assert_eq!(
        err(&["v:1 2 3", "v 0:5"]),
        QError::parse("unexpected : after expression")
    );
}

#[test]
fn iassign_evaluation_order() {
    // the value first, then the index (right to left)
    assert_eq!(
        err(&["v:1 2 3", "v[i:1]:i"]),
        QError::Undefined("i".into()),
        "the value `i` is evaluated before the index sets it"
    );
    assert_eq!(show(&["v:1 2 3", "i:0", "v[i:1]:i", "v"]), "1 0 3");
    assert_eq!(show(&["v:1 2 3", "i:0", "v[i]:i:2", "v"]), "1 2 2");
    // the value assignment is done before the target is read
    assert_eq!(show(&["v:1 2 3", "v[0 1]:v:7 8", "v"]), "7 8");
    assert_eq!(err(&["v:1 2 3", "v[0]:v:7 8"]), QError::Type);
    assert_eq!(show(&["v:1 2 3", "v[1]:v[0]:7", "v"]), "7 7 3");
}

#[test]
fn iassign_in_an_expression() {
    // the value of the statement is what the index now reads, wherever it stands
    assert_eq!(session(&["v:1 2 3", "1+v[0]:5"]), Ok(long(6)));
    assert_eq!(show(&["v:1 2 3", "1+v[0]:5", "v"]), "5 2 3");
    assert_eq!(session(&["v:1 2 3", "1+v[0 1]:5 6"]), Ok(longs(&[6, 7])));
    assert_eq!(session(&["v:1 2 3", "x:v[0 2]:9", "x"]), Ok(longs(&[9, 9])));
    assert_eq!(
        session(&["v:1 2 3", "x:(v[0 2]:9)", "x"]),
        Ok(longs(&[9, 9]))
    );
    // with a duplicate index the value is what v holds afterwards, not what was given
    assert_eq!(session(&["v:1 2 3", "v[0 0]:1 2"]), Ok(longs(&[2, 2])));
    assert_eq!(session(&["v:1 2 3", "v[1 0]:5 6"]), Ok(longs(&[5, 6])));
    assert_eq!(session(&["v:1 2 3", "v[01b]:7 8"]), Ok(longs(&[7, 8])));
    // the value takes the rest of the line
    assert_eq!(show(&["v:1 2 3", "v[0]:5+1", "v"]), "6 2 3");
    assert_eq!(show(&["v:1 2 3", "v[0]:-5", "v"]), "-5 2 3");
    assert_eq!(show(&["v:1 2 3", "- v[0]:5"]), "-5");
    // an assignment in the index
    assert_eq!(show(&["v:1 2 3", "v[j:2]:9", "j",]), "2");
    // after a verb, and as the first operand of a chain
    assert_eq!(show(&["v:1 2 3", "w:v[0]:5", "w"]), "5");
    assert_eq!(show(&["v:1 2 3", "10*v[2]:4", "v"]), "1 2 4");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    // after v[i]:x with a valid i, v i is x and every other item is untouched
    #[test]
    fn iassign_atom_reads_back(
        v in proptest::collection::vec(-100i64..100, 1..20),
        pick in 0usize..1000,
        x in -100i64..100,
    ) {
        let i = pick % v.len();
        let mut it = Interpreter::new();
        it.set("v", longs(&v));
        prop_assert_eq!(it.eval_line(&format!("v[{i}]:{x}")).unwrap(), Some(long(x)));
        let mut want = v.clone();
        want[i] = x;
        prop_assert_eq!(it.get("v"), Some(&longs(&want)));
        prop_assert_eq!(it.eval_line(&format!("v[{i}]")).unwrap(), Some(long(x)));
    }

    // vector index and value against a plain model: pairwise, last duplicate wins
    #[test]
    fn iassign_vector_matches_model(
        v in proptest::collection::vec(-100i64..100, 1..12),
        picks in proptest::collection::vec(0usize..1000, 1..8),
        xs in proptest::collection::vec(-100i64..100, 8),
        broadcast in any::<bool>(),
    ) {
        let idx: Vec<usize> = picks.iter().map(|p| p % v.len()).collect();
        let mut it = Interpreter::new();
        it.set("v", longs(&v));
        it.set("i", longs(&idx.iter().map(|&k| k as i64).collect::<Vec<_>>()));
        let given: Vec<i64> = xs[..idx.len()].to_vec();
        let mut want = v.clone();
        if broadcast {
            it.set("x", long(given[0]));
            for &k in &idx { want[k] = given[0]; }
        } else {
            it.set("x", longs(&given));
            for (&k, &g) in idx.iter().zip(&given) { want[k] = g; }
        }
        it.eval_line("v[i]:x").unwrap();
        prop_assert_eq!(it.get("v"), Some(&longs(&want)));
    }

    // an alias never sees the change, whatever the index and value
    #[test]
    fn iassign_never_mutates_an_alias(
        v in proptest::collection::vec(-100i64..100, 1..12),
        pick in 0usize..1000,
        x in -100i64..100,
    ) {
        let mut it = Interpreter::new();
        it.set("v", longs(&v));
        it.eval_line("w:v").unwrap();
        let k = pick % v.len();
        it.eval_line(&format!("v[{k}]:{x}")).unwrap();
        prop_assert_eq!(it.get("w"), Some(&longs(&v)));
        it.eval_line(&format!("w[{k}]:{}", x + 1000)).unwrap();
        prop_assert_eq!(it.get("v").map(|a| a.to_string()), {
            let mut want = v.clone();
            want[k] = x;
            Some(longs(&want).to_string())
        });
    }

    // out of range is always 'length and never changes the vector
    #[test]
    fn iassign_out_of_range_never_changes(
        v in proptest::collection::vec(-100i64..100, 0..10),
        off in 0i64..50,
        neg in any::<bool>(),
    ) {
        let k = if neg { -1 - off } else { v.len() as i64 + off };
        let mut it = Interpreter::new();
        it.set("v", longs(&v));
        prop_assert_eq!(it.eval_line(&format!("v[{k}]:1")), Err(QError::Length));
        prop_assert_eq!(it.get("v"), Some(&longs(&v)));
    }
}

#[test]
fn iassign_library_values() {
    let mut i = Interpreter::new();
    i.set("v", longs(&[1, 2, 3]));
    // an atom with no column type (a date) is a type error, not a silent no-op
    let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
    i.set("d", Value::Atom(Atom::Date(date)));
    assert_eq!(i.eval_line("v[0]:d"), Err(QError::Type));
    assert_eq!(i.eval_line("v[0 1]:d"), Err(QError::Type));
    // an empty general list is an empty index: nothing changes
    i.set("e", Value::List(Rc::new(vec![])));
    assert_eq!(
        i.eval_line("v[e]:5").map(|v| v.unwrap().to_string()),
        Ok("()".into())
    );
    assert_eq!(i.get("v"), Some(&longs(&[1, 2, 3])));
    // a non-empty general list is not an index
    i.set("e", Value::List(Rc::new(vec![long(0), sym("a")])));
    assert_eq!(i.eval_line("v[e]:5"), Err(QError::Type));
}

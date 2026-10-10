use oxidedb::language::ast::{Expr, Verb};
use oxidedb::language::ops::dyad;
use oxidedb::types::column::Column;
use oxidedb::types::sym::Sym;
use oxidedb::{Atom, Interpreter, QError, Value};
use proptest::prelude::*;
use std::rc::Rc;

const NULL: i64 = i64::MIN;
const NAN: f64 = f64::NAN;
const CMP: [Verb; 6] = [
    Verb::Equal,
    Verb::NotEqual,
    Verb::Less,
    Verb::LessEqual,
    Verb::Greater,
    Verb::GreaterEqual,
];

fn l(n: i64) -> Value {
    Value::Atom(Atom::Integer(n))
}
fn f(x: f64) -> Value {
    Value::Atom(Atom::Float(x))
}
fn b(x: bool) -> Value {
    Value::Atom(Atom::Boolean(x))
}
fn s(name: &str) -> Value {
    Value::Atom(Atom::Symbol(Sym::intern(name)))
}
fn c(x: char) -> Value {
    Value::Atom(Atom::Character(x))
}
fn lv(xs: &[i64]) -> Value {
    Value::Vector(Rc::new(Column::Long(xs.to_vec())))
}
fn fv(xs: &[f64]) -> Value {
    Value::Vector(Rc::new(Column::Float(xs.to_vec())))
}
fn bv(xs: &[bool]) -> Value {
    Value::Vector(Rc::new(Column::Bool(xs.to_vec())))
}
fn sv(names: &[&str]) -> Value {
    Value::Vector(Rc::new(Column::Sym(
        names.iter().map(|n| Sym::intern(n)).collect(),
    )))
}
fn cv(text: &str) -> Value {
    Value::Vector(Rc::new(Column::Char(text.chars().collect())))
}
fn list(items: Vec<Value>) -> Value {
    Value::List(Rc::new(items))
}
fn eq(x: &Value, y: &Value) -> Result<Value, QError> {
    dyad(Verb::Equal, x, y)
}
fn ne(x: &Value, y: &Value) -> Result<Value, QError> {
    dyad(Verb::NotEqual, x, y)
}
fn lt(x: &Value, y: &Value) -> Result<Value, QError> {
    dyad(Verb::Less, x, y)
}
fn le(x: &Value, y: &Value) -> Result<Value, QError> {
    dyad(Verb::LessEqual, x, y)
}
fn gt(x: &Value, y: &Value) -> Result<Value, QError> {
    dyad(Verb::Greater, x, y)
}
fn ge(x: &Value, y: &Value) -> Result<Value, QError> {
    dyad(Verb::GreaterEqual, x, y)
}

/// `[=, <>, <, <=, >, >=]` results for `x op y`, all atoms.
fn row(x: &Value, y: &Value) -> [bool; 6] {
    CMP.map(|v| match dyad(v, x, y) {
        Ok(Value::Atom(Atom::Boolean(t))) => t,
        other => panic!("{v:?} {x:?} {y:?}: {other:?}"),
    })
}
const LESS: [bool; 6] = [false, true, true, true, false, false];
const SAME: [bool; 6] = [true, false, false, true, false, true];
const MORE: [bool; 6] = [false, true, false, false, true, true];

#[test]
fn cmp_atoms() {
    assert_eq!(lt(&l(1), &l(2)), Ok(b(true)));
    assert_eq!(eq(&l(1), &f(1.0)), Ok(b(true)));
    // every operator, all three orderings, every numeric type pair
    let pairs: [(Value, Value, Value); 4] = [
        (l(1), l(2), l(3)),
        (f(1.0), f(2.0), f(3.0)),
        (l(1), f(2.0), l(3)),
        (f(1.0), l(2), f(3.0)),
    ];
    for (lo, mid, hi) in pairs {
        assert_eq!(row(&lo, &mid), LESS, "{lo:?} {mid:?}");
        assert_eq!(row(&mid, &lo), MORE, "{mid:?} {lo:?}");
        assert_eq!(row(&mid, &mid), SAME, "{mid:?}");
        assert_eq!(row(&mid, &hi), LESS, "{mid:?} {hi:?}");
        assert_eq!(row(&hi, &mid), MORE, "{hi:?} {mid:?}");
    }
    assert_eq!(row(&b(false), &b(true)), LESS);
    assert_eq!(row(&b(true), &b(false)), MORE);
    assert_eq!(row(&b(true), &b(true)), SAME);
    // bool, long and float compare by value
    assert_eq!(row(&b(true), &l(1)), SAME);
    assert_eq!(row(&l(1), &b(true)), SAME);
    assert_eq!(row(&b(true), &f(1.0)), SAME);
    assert_eq!(row(&f(1.0), &b(true)), SAME);
    assert_eq!(row(&b(true), &l(2)), LESS);
    assert_eq!(row(&l(2), &b(true)), MORE);
    assert_eq!(row(&b(false), &f(-0.5)), MORE);
    assert_eq!(row(&f(-0.5), &b(false)), LESS);
    assert_eq!(row(&l(-1), &b(false)), LESS);
    // longs compare exactly, not through floats
    assert_eq!(row(&l((1 << 53) + 1), &l(1 << 53)), MORE);
    assert_eq!(row(&l(i64::MAX), &l(i64::MAX - 1)), MORE);
    // infinities
    assert_eq!(row(&f(f64::INFINITY), &l(i64::MAX)), MORE);
    assert_eq!(row(&f(f64::NEG_INFINITY), &l(i64::MIN + 1)), LESS);
    assert_eq!(row(&f(0.0), &f(-0.0)), SAME);
}

#[test]
fn cmp_vector_atom() {
    assert_eq!(gt(&lv(&[1, 2, 3]), &l(1)), Ok(bv(&[false, true, true])));
    // operand order matters
    assert_eq!(gt(&l(1), &lv(&[1, 2, 3])), Ok(bv(&[false, false, false])));
    assert_eq!(lt(&l(1), &lv(&[1, 2, 3])), Ok(bv(&[false, true, true])));
    assert_eq!(lt(&lv(&[1, 2, 3]), &l(2)), Ok(bv(&[true, false, false])));
    assert_eq!(le(&lv(&[1, 2, 3]), &l(2)), Ok(bv(&[true, true, false])));
    assert_eq!(le(&l(2), &lv(&[1, 2, 3])), Ok(bv(&[false, true, true])));
    assert_eq!(ge(&lv(&[1, 2, 3]), &l(2)), Ok(bv(&[false, true, true])));
    assert_eq!(ge(&l(2), &lv(&[1, 2, 3])), Ok(bv(&[true, true, false])));
    assert_eq!(eq(&lv(&[1, 2, 1]), &l(1)), Ok(bv(&[true, false, true])));
    assert_eq!(eq(&l(1), &lv(&[1, 2, 1])), Ok(bv(&[true, false, true])));
    assert_eq!(ne(&lv(&[1, 2, 1]), &l(1)), Ok(bv(&[false, true, false])));
    assert_eq!(ne(&l(1), &lv(&[1, 2, 1])), Ok(bv(&[false, true, false])));
    // across types
    assert_eq!(eq(&lv(&[1, 2]), &f(1.0)), Ok(bv(&[true, false])));
    assert_eq!(eq(&f(1.0), &lv(&[1, 2])), Ok(bv(&[true, false])));
    assert_eq!(lt(&fv(&[0.5, 1.5]), &l(1)), Ok(bv(&[true, false])));
    assert_eq!(lt(&l(1), &fv(&[0.5, 1.5])), Ok(bv(&[false, true])));
    assert_eq!(eq(&bv(&[true, false]), &l(1)), Ok(bv(&[true, false])));
    assert_eq!(eq(&l(1), &bv(&[true, false])), Ok(bv(&[true, false])));
    assert_eq!(gt(&bv(&[true, false]), &f(0.5)), Ok(bv(&[true, false])));
}

#[test]
fn cmp_vector_vector() {
    let x = lv(&[1, 5, 3]);
    let y = lv(&[2, 5, 1]);
    assert_eq!(eq(&x, &y), Ok(bv(&[false, true, false])));
    assert_eq!(ne(&x, &y), Ok(bv(&[true, false, true])));
    assert_eq!(lt(&x, &y), Ok(bv(&[true, false, false])));
    assert_eq!(le(&x, &y), Ok(bv(&[true, true, false])));
    assert_eq!(gt(&x, &y), Ok(bv(&[false, false, true])));
    assert_eq!(ge(&x, &y), Ok(bv(&[false, true, true])));
    // long against float, bool against long, both orders
    assert_eq!(lt(&lv(&[1, 2]), &fv(&[1.5, 1.5])), Ok(bv(&[true, false])));
    assert_eq!(lt(&fv(&[1.5, 1.5]), &lv(&[1, 2])), Ok(bv(&[false, true])));
    assert_eq!(
        eq(&bv(&[true, false]), &lv(&[1, 1])),
        Ok(bv(&[true, false]))
    );
    assert_eq!(
        eq(&bv(&[true, false]), &bv(&[true, true])),
        Ok(bv(&[true, false]))
    );
}

#[test]
fn cmp_length_mismatch() {
    for verb in CMP {
        let e = Err(QError::Length);
        assert_eq!(dyad(verb, &lv(&[1, 2]), &lv(&[1, 2, 3])), e);
        assert_eq!(dyad(verb, &lv(&[1, 2, 3]), &lv(&[1, 2])), e);
        assert_eq!(dyad(verb, &lv(&[]), &lv(&[1])), e);
        assert_eq!(dyad(verb, &lv(&[1]), &lv(&[])), e);
        assert_eq!(dyad(verb, &fv(&[1.0]), &lv(&[1, 2])), e);
        assert_eq!(dyad(verb, &sv(&["a"]), &sv(&["a", "b"])), e);
        assert_eq!(dyad(verb, &cv("ab"), &cv("a")), e);
        assert_eq!(dyad(verb, &bv(&[true]), &lv(&[1, 2])), e);
        let two = list(vec![l(1), lv(&[2, 3])]);
        assert_eq!(dyad(verb, &two, &lv(&[1, 2, 3])), e);
        assert_eq!(dyad(verb, &lv(&[1, 2, 3]), &two), e);
        assert_eq!(dyad(verb, &two, &list(vec![l(1)])), e);
    }
}

#[test]
fn cmp_null_equals_null() {
    assert_eq!(eq(&l(NULL), &l(NULL)), Ok(b(true)));
    assert_eq!(eq(&f(NAN), &f(NAN)), Ok(b(true)));
    assert_eq!(ne(&l(NULL), &l(NULL)), Ok(b(false)));
    assert_eq!(ne(&f(NAN), &f(NAN)), Ok(b(false)));
    // the two kinds of numeric null are the same null
    assert_eq!(eq(&l(NULL), &f(NAN)), Ok(b(true)));
    assert_eq!(eq(&f(NAN), &l(NULL)), Ok(b(true)));
    // a null is not any other number
    assert_eq!(eq(&l(NULL), &l(0)), Ok(b(false)));
    assert_eq!(eq(&l(0), &l(NULL)), Ok(b(false)));
    assert_eq!(eq(&f(NAN), &f(0.0)), Ok(b(false)));
    assert_eq!(eq(&f(0.0), &f(NAN)), Ok(b(false)));
    assert_eq!(eq(&l(NULL), &f(f64::NEG_INFINITY)), Ok(b(false)));
    assert_eq!(ne(&l(0), &l(NULL)), Ok(b(true)));
    assert_eq!(le(&l(NULL), &l(NULL)), Ok(b(true)));
    assert_eq!(ge(&f(NAN), &f(NAN)), Ok(b(true)));
    assert_eq!(lt(&l(NULL), &l(NULL)), Ok(b(false)));
    assert_eq!(gt(&f(NAN), &f(NAN)), Ok(b(false)));
    // in vectors
    assert_eq!(eq(&lv(&[NULL, 1]), &l(NULL)), Ok(bv(&[true, false])));
    assert_eq!(eq(&fv(&[NAN, 1.0]), &f(NAN)), Ok(bv(&[true, false])));
    assert_eq!(
        eq(&fv(&[NAN, 1.0]), &fv(&[NAN, NAN])),
        Ok(bv(&[true, false]))
    );
    assert_eq!(
        eq(&lv(&[NULL, 1]), &fv(&[NAN, NAN])),
        Ok(bv(&[true, false]))
    );
    assert_eq!(
        eq(&fv(&[NAN, NAN]), &lv(&[NULL, 1])),
        Ok(bv(&[true, false]))
    );
    // symbol and character nulls equal themselves too
    assert_eq!(eq(&s(""), &s("")), Ok(b(true)));
    assert_eq!(eq(&c(' '), &c(' ')), Ok(b(true)));
}

#[test]
fn cmp_null_sorts_lowest() {
    assert_eq!(lt(&l(NULL), &l(1)), Ok(b(true)));
    assert_eq!(lt(&l(NULL), &l(-5)), Ok(b(true)));
    assert_eq!(lt(&l(NULL), &l(i64::MIN + 1)), Ok(b(true)));
    assert_eq!(gt(&l(1), &l(NULL)), Ok(b(true)));
    assert_eq!(gt(&l(NULL), &l(1)), Ok(b(false)));
    assert_eq!(le(&l(NULL), &l(1)), Ok(b(true)));
    assert_eq!(ge(&l(1), &l(NULL)), Ok(b(true)));
    assert_eq!(ge(&l(NULL), &l(1)), Ok(b(false)));
    assert_eq!(lt(&f(NAN), &f(1.0)), Ok(b(true)));
    assert_eq!(lt(&f(1.0), &f(NAN)), Ok(b(false)));
    assert_eq!(gt(&f(1.0), &f(NAN)), Ok(b(true)));
    assert_eq!(gt(&f(NAN), &f(1.0)), Ok(b(false)));
    // below negative infinity
    // ponytail: verify against q: 0n<-0w is 1b in q 4.x as implemented here
    assert_eq!(lt(&f(NAN), &f(f64::NEG_INFINITY)), Ok(b(true)));
    assert_eq!(gt(&f(f64::NEG_INFINITY), &f(NAN)), Ok(b(true)));
    assert_eq!(lt(&l(NULL), &f(f64::NEG_INFINITY)), Ok(b(true)));
    assert_eq!(gt(&f(f64::NEG_INFINITY), &l(NULL)), Ok(b(true)));
    // mixed null kinds against ordinary numbers
    assert_eq!(lt(&l(NULL), &f(0.5)), Ok(b(true)));
    assert_eq!(lt(&f(NAN), &l(-7)), Ok(b(true)));
    assert_eq!(gt(&l(-7), &f(NAN)), Ok(b(true)));
    assert_eq!(gt(&f(0.5), &l(NULL)), Ok(b(true)));
    assert_eq!(lt(&l(NULL), &b(false)), Ok(b(true)));
    assert_eq!(lt(&f(NAN), &b(false)), Ok(b(true)));
    // vectors
    assert_eq!(lt(&lv(&[NULL, 1, 2]), &l(1)), Ok(bv(&[true, false, false])));
    assert_eq!(gt(&l(1), &lv(&[NULL, 1, 2])), Ok(bv(&[true, false, false])));
    assert_eq!(lt(&fv(&[NAN, 1.0]), &l(1)), Ok(bv(&[true, false])));
    assert_eq!(
        lt(&fv(&[NAN, 1.0]), &fv(&[0.0, 0.0])),
        Ok(bv(&[true, false]))
    );
    assert_eq!(
        lt(&lv(&[NULL, 1]), &fv(&[0.0, 0.0])),
        Ok(bv(&[true, false]))
    );
    assert_eq!(
        gt(&fv(&[0.0, 0.0]), &lv(&[NULL, 1])),
        Ok(bv(&[true, false]))
    );
    // the null symbol is the lowest symbol
    assert_eq!(lt(&s(""), &s("a")), Ok(b(true)));
    assert_eq!(gt(&s("a"), &s("")), Ok(b(true)));
}

#[test]
fn cmp_symbols() {
    assert_eq!(eq(&s("a"), &s("a")), Ok(b(true)));
    assert_eq!(eq(&s("a"), &s("b")), Ok(b(false)));
    assert_eq!(ne(&s("a"), &s("b")), Ok(b(true)));
    assert_eq!(row(&s("a"), &s("b")), LESS);
    assert_eq!(row(&s("b"), &s("a")), MORE);
    assert_eq!(row(&s("a"), &s("a")), SAME);
    // lexicographic by name, not by interning order
    let _ = (s("zzz"), s("aaa"));
    assert_eq!(row(&s("aaa"), &s("zzz")), LESS);
    assert_eq!(row(&s("zzz"), &s("aaa")), MORE);
    assert_eq!(row(&s("ab"), &s("b")), LESS);
    assert_eq!(row(&s("b"), &s("ab")), MORE);
    // a prefix sorts before its extension
    assert_eq!(row(&s("a"), &s("ab")), LESS);
    assert_eq!(row(&s("ab"), &s("a")), MORE);
    // byte order: upper case before lower case
    assert_eq!(row(&s("A"), &s("a")), LESS);
    // vectors
    assert_eq!(eq(&sv(&["a", "b"]), &s("a")), Ok(bv(&[true, false])));
    assert_eq!(eq(&s("a"), &sv(&["a", "b"])), Ok(bv(&[true, false])));
    assert_eq!(lt(&sv(&["a", "b"]), &s("b")), Ok(bv(&[true, false])));
    assert_eq!(
        lt(&s("b"), &sv(&["a", "b", "c"])),
        Ok(bv(&[false, false, true]))
    );
    assert_eq!(
        ge(&sv(&["a", "b"]), &sv(&["b", "b"])),
        Ok(bv(&[false, true]))
    );
    assert_eq!(
        ne(&sv(&["a", "b"]), &sv(&["b", "b"])),
        Ok(bv(&[true, false]))
    );
    assert_eq!(
        gt(&sv(&["c", "b"]), &sv(&["b", "b"])),
        Ok(bv(&[true, false]))
    );
    assert_eq!(
        le(&sv(&["c", "b"]), &sv(&["b", "b"])),
        Ok(bv(&[false, true]))
    );
}

#[test]
fn cmp_chars() {
    assert_eq!(row(&c('a'), &c('b')), LESS);
    assert_eq!(row(&c('b'), &c('a')), MORE);
    assert_eq!(row(&c('a'), &c('a')), SAME);
    assert_eq!(row(&c('Z'), &c('a')), LESS);
    assert_eq!(eq(&cv("abc"), &c('b')), Ok(bv(&[false, true, false])));
    assert_eq!(eq(&c('b'), &cv("abc")), Ok(bv(&[false, true, false])));
    assert_eq!(lt(&cv("abc"), &c('b')), Ok(bv(&[true, false, false])));
    assert_eq!(lt(&c('b'), &cv("abc")), Ok(bv(&[false, false, true])));
    assert_eq!(ge(&cv("abc"), &cv("bbb")), Ok(bv(&[false, true, true])));
    assert_eq!(ne(&cv("abc"), &cv("abd")), Ok(bv(&[false, false, true])));
    assert_eq!(le(&cv("abc"), &cv("bbb")), Ok(bv(&[true, true, false])));
    assert_eq!(gt(&cv("abc"), &cv("bbb")), Ok(bv(&[false, false, true])));
}

#[test]
fn cmp_symbol_vs_long_is_type() {
    let date = Value::Atom(Atom::NullDate);
    for verb in CMP {
        let e = Err(QError::Type);
        for (x, y) in [
            (s("a"), l(1)),
            (s("a"), f(1.0)),
            (s("a"), b(true)),
            (c('a'), l(1)),
            (c('a'), f(1.0)),
            (c('a'), b(true)),
            (s("a"), c('a')),
            (date.clone(), l(1)),
            (date.clone(), date.clone()),
            (sv(&["a"]), l(1)),
            (sv(&["a"]), lv(&[1])),
            (sv(&["a"]), cv("a")),
            (cv("a"), lv(&[1])),
            (cv("a"), l(1)),
            (s("a"), lv(&[1])),
            (s("a"), cv("a")),
            (c('a'), sv(&["a"])),
            (bv(&[true]), sv(&["a"])),
            (fv(&[1.0]), cv("a")),
            // empty vectors still have a type
            (sv(&[]), l(1)),
            (cv(""), l(1)),
            (sv(&[]), cv("")),
            (lv(&[]), s("a")),
            (fv(&[]), c('a')),
            (bv(&[]), s("a")),
        ] {
            assert_eq!(dyad(verb, &x, &y), e, "{verb:?} {x:?} {y:?}");
            assert_eq!(dyad(verb, &y, &x), e, "{verb:?} {y:?} {x:?}");
        }
        // inside a general list
        assert_eq!(dyad(verb, &list(vec![l(1), sv(&["a"])]), &l(1)), e);
    }
}

#[test]
fn cmp_general_list_recurses() {
    let nested = list(vec![l(1), lv(&[2, 3])]);
    assert_eq!(
        gt(&nested, &l(1)),
        Ok(list(vec![b(false), bv(&[true, true])]))
    );
    assert_eq!(
        lt(&l(1), &nested),
        Ok(list(vec![b(false), bv(&[true, true])]))
    );
    assert_eq!(
        le(&nested, &l(2)),
        Ok(list(vec![b(true), bv(&[true, false])]))
    );
    assert_eq!(
        eq(&nested, &list(vec![l(1), lv(&[2, 4])])),
        Ok(list(vec![b(true), bv(&[true, false])]))
    );
    assert_eq!(
        lt(&nested, &lv(&[2, 3])),
        Ok(list(vec![b(true), bv(&[true, false])]))
    );
    assert_eq!(
        gt(&lv(&[2, 3]), &nested),
        Ok(list(vec![b(true), bv(&[true, false])]))
    );
    // normalised: a hand-built list of atoms gives a Bool vector
    assert_eq!(gt(&list(vec![l(1), l(5)]), &l(2)), Ok(bv(&[false, true])));
    assert_eq!(gt(&list(vec![]), &l(2)), Ok(list(vec![])));
    assert_eq!(gt(&nested, &lv(&[1, 2, 3])), Err(QError::Length));
}

#[test]
fn cmp_list_results_collapse_to_bool_vectors() {
    let mixed = || list(vec![l(1), s("a")]);
    assert_eq!(eq(&mixed(), &mixed()), Ok(bv(&[true, true])));
    assert_eq!(ne(&mixed(), &mixed()), Ok(bv(&[false, false])));
    assert_eq!(
        lt(&list(vec![l(1), l(2)]), &lv(&[2, 2])),
        Ok(bv(&[true, false]))
    );
    assert_eq!(
        lt(&lv(&[1, 2]), &list(vec![l(2), l(2)])),
        Ok(bv(&[true, false]))
    );
    assert_eq!(gt(&list(vec![l(1), l(2)]), &l(1)), Ok(bv(&[false, true])));
    assert_eq!(gt(&l(2), &list(vec![l(1), l(2)])), Ok(bv(&[true, false])));
}

// Decisions where O differs from q or follows its conversion rule.
#[test]
fn cmp_precision_decisions() {
    // a long against a float converts to float first, as q does
    assert_eq!(
        eq(&l(9007199254740993), &f(9007199254740992.0)),
        Ok(b(true))
    );
    assert_eq!(
        eq(&f(9007199254740992.0), &l(9007199254740993)),
        Ok(b(true))
    );
    // long against long stays exact
    assert_eq!(eq(&l(9007199254740993), &l(9007199254740992)), Ok(b(false)));
    // float equality is exact in O (q is tolerant)
    assert_eq!(eq(&f(0.1 + 0.2), &f(0.3)), Ok(b(false)));
    assert_eq!(gt(&f(0.1 + 0.2), &f(0.3)), Ok(b(true)));
    // a character is not a number in O (q compares by code)
    assert_eq!(eq(&c('a'), &l(97)), Err(QError::Type));
}

#[test]
fn cmp_empty_vectors() {
    for verb in CMP {
        assert_eq!(dyad(verb, &lv(&[]), &l(1)), Ok(bv(&[])));
        assert_eq!(dyad(verb, &l(1), &lv(&[])), Ok(bv(&[])));
        assert_eq!(dyad(verb, &lv(&[]), &lv(&[])), Ok(bv(&[])));
        assert_eq!(dyad(verb, &fv(&[]), &lv(&[])), Ok(bv(&[])));
        assert_eq!(dyad(verb, &sv(&[]), &s("a")), Ok(bv(&[])));
        assert_eq!(dyad(verb, &cv(""), &c('a')), Ok(bv(&[])));
        assert_eq!(dyad(verb, &sv(&[]), &sv(&[])), Ok(bv(&[])));
        assert_eq!(dyad(verb, &bv(&[]), &bv(&[])), Ok(bv(&[])));
    }
}

fn node(operator: Verb, left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        left: Box::new(left),
        operator,
        right: Box::new(right),
    }
}

// The lexer does not produce comparison tokens yet; build the AST by hand.
#[test]
fn cmp_through_the_interpreter() {
    let mut i = Interpreter::new();
    i.set("v", lv(&[1, 2, 3]));
    i.set("w", lv(&[3, 2, 1]));
    let var = |n: &str| Expr::Symbol(n.into());
    let int = |n: i64| Expr::Atom(Atom::Integer(n));
    assert_eq!(
        i.evaluate(node(Verb::Greater, var("v"), int(1))),
        Ok(bv(&[false, true, true]))
    );
    assert_eq!(
        i.evaluate(node(Verb::Less, var("v"), var("w"))),
        Ok(bv(&[true, false, false]))
    );
    assert_eq!(
        i.evaluate(node(Verb::Equal, int(1), Expr::Atom(Atom::Float(1.0)))),
        Ok(b(true))
    );
    assert_eq!(
        i.evaluate(node(Verb::LessEqual, var("v"), int(NULL))),
        Ok(bv(&[false, false, false]))
    );
    assert_eq!(
        i.evaluate(node(
            Verb::NotEqual,
            var("v"),
            Expr::Atom(Atom::Symbol(Sym::intern("a")))
        )),
        Err(QError::Type)
    );
    // comparison results feed arithmetic: booleans count as 0 and 1
    assert_eq!(
        i.evaluate(node(
            Verb::Add,
            node(Verb::Greater, var("v"), int(1)),
            int(10)
        )),
        Ok(lv(&[10, 11, 11]))
    );
}

fn num() -> impl Strategy<Value = Value> {
    prop_oneof![
        prop_oneof![-3i64..3, Just(NULL), Just(NULL + 1), Just(i64::MAX)].prop_map(l),
        prop_oneof![
            (-3i32..3).prop_map(|n| f64::from(n) / 2.0),
            Just(NAN),
            Just(f64::INFINITY),
            Just(f64::NEG_INFINITY)
        ]
        .prop_map(f),
        any::<bool>().prop_map(b),
    ]
}

fn flag(r: Result<Value, QError>) -> bool {
    match r {
        Ok(Value::Atom(Atom::Boolean(t))) => t,
        other => panic!("{other:?}"),
    }
}

proptest! {
    // Exactly one of <, =, > holds, and the other operators are their combinations.
    #[test]
    fn cmp_prop_trichotomy(x in num(), y in num()) {
        let (e, n, lo, loe, hi, hie) = (
            flag(eq(&x, &y)), flag(ne(&x, &y)), flag(lt(&x, &y)),
            flag(le(&x, &y)), flag(gt(&x, &y)), flag(ge(&x, &y)),
        );
        prop_assert_eq!(u8::from(lo) + u8::from(e) + u8::from(hi), 1);
        prop_assert_eq!(n, !e);
        prop_assert_eq!(loe, lo || e);
        prop_assert_eq!(hie, hi || e);
        // swapping operands mirrors the ordering
        prop_assert_eq!(lo, flag(gt(&y, &x)));
        prop_assert_eq!(loe, flag(ge(&y, &x)));
        prop_assert_eq!(e, flag(eq(&y, &x)));
    }

    #[test]
    fn cmp_prop_vector_atom_is_mapped_atom_op(
        xs in prop::collection::vec(-3i64..3, 1..8), a in -3i64..3, k in 0usize..6
    ) {
        let verb = CMP[k];
        let want: Vec<Value> = xs.iter().map(|&x| dyad(verb, &l(x), &l(a)).unwrap()).collect();
        prop_assert_eq!(dyad(verb, &lv(&xs), &l(a)), Ok(Value::from_items(want)));
        let want: Vec<Value> = xs.iter().map(|&x| dyad(verb, &l(a), &l(x)).unwrap()).collect();
        prop_assert_eq!(dyad(verb, &l(a), &lv(&xs)), Ok(Value::from_items(want)));
    }

    #[test]
    fn cmp_prop_broadcast_preserves_length(
        xs in prop::collection::vec(-3i64..3, 0..16), a in -3i64..3, k in 0usize..6
    ) {
        for r in [dyad(CMP[k], &lv(&xs), &l(a)), dyad(CMP[k], &l(a), &lv(&xs))] {
            match r.unwrap() {
                Value::Vector(col) => {
                    prop_assert_eq!(col.len(), xs.len());
                    prop_assert_eq!(col.type_code(), 1);
                }
                other => prop_assert!(false, "expected a Bool vector, got {other:?}"),
            }
        }
    }
}

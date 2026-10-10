//! `x,y` through the kernel. Every expected value was checked against q 4.1.
use oxidedb::language::ast::Verb;
use oxidedb::language::ops::dyad;
use oxidedb::types::column::{Column, MAX_ELEMS};
use oxidedb::types::sym::Sym;
use oxidedb::{Atom, QError, Value};
use proptest::prelude::*;
use std::rc::Rc;

fn l(n: i64) -> Value {
    Value::Atom(Atom::Integer(n))
}
fn f(x: f64) -> Value {
    Value::Atom(Atom::Float(x))
}
fn b(x: bool) -> Value {
    Value::Atom(Atom::Boolean(x))
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
fn cv(s: &str) -> Value {
    Value::Vector(Rc::new(Column::Char(s.chars().collect())))
}
fn sym(s: &str) -> Value {
    Value::Atom(Atom::Symbol(Sym::intern(s)))
}
fn sv(names: &[&str]) -> Value {
    Value::Vector(Rc::new(Column::Sym(
        names.iter().map(|n| Sym::intern(n)).collect(),
    )))
}
fn list(items: Vec<Value>) -> Value {
    Value::List(Rc::new(items))
}
fn join(x: &Value, y: &Value) -> Result<Value, QError> {
    dyad(Verb::Join, x, y)
}

#[test]
fn join_same_type_preserves() {
    assert_eq!(join(&lv(&[1, 2]), &lv(&[3, 4])), Ok(lv(&[1, 2, 3, 4])));
    assert_eq!(
        join(&fv(&[1.5]), &fv(&[2.5, 3.5])),
        Ok(fv(&[1.5, 2.5, 3.5]))
    );
    assert_eq!(join(&bv(&[true]), &bv(&[false])), Ok(bv(&[true, false])));
    assert_eq!(join(&cv("ab"), &cv("cd")), Ok(cv("abcd")));
    assert_eq!(
        join(&sv(&["a"]), &sv(&["b", "c"])),
        Ok(sv(&["a", "b", "c"]))
    );
}

#[test]
fn join_atom_vector() {
    assert_eq!(join(&l(1), &lv(&[2, 3])), Ok(lv(&[1, 2, 3])));
    assert_eq!(join(&lv(&[1, 2]), &l(3)), Ok(lv(&[1, 2, 3])));
    assert_eq!(join(&l(1), &l(2)), Ok(lv(&[1, 2])));
    assert_eq!(join(&f(1.5), &f(0.5)), Ok(fv(&[1.5, 0.5])));
    assert_eq!(join(&b(true), &b(true)), Ok(bv(&[true, true])));
    assert_eq!(join(&f(1.5), &fv(&[2.5])), Ok(fv(&[1.5, 2.5])));
    // nulls are ordinary members
    assert_eq!(
        join(&lv(&[1, 2, 3]), &l(i64::MIN)),
        Ok(lv(&[1, 2, 3, i64::MIN]))
    );
    assert_eq!(join(&f(1.5), &f(f64::NAN)), Ok(fv(&[1.5, f64::NAN])));
}

#[test]
fn join_mixed_types_general() {
    // no promotion: 1,2.5 is (1;2.5), 1 2,3.0 is (1;2;3f)
    assert_eq!(join(&l(1), &f(2.5)), Ok(list(vec![l(1), f(2.5)])));
    assert_eq!(
        join(&lv(&[1, 2]), &f(3.0)),
        Ok(list(vec![l(1), l(2), f(3.0)]))
    );
    assert_eq!(
        join(&lv(&[1, 2]), &c('a')),
        Ok(list(vec![l(1), l(2), c('a')]))
    );
    assert_eq!(join(&lv(&[1]), &fv(&[2.0])), Ok(list(vec![l(1), f(2.0)])));
    assert_eq!(join(&b(true), &l(1)), Ok(list(vec![b(true), l(1)])));
    assert_eq!(join(&c('a'), &l(1)), Ok(list(vec![c('a'), l(1)])));
    assert_eq!(
        join(&lv(&[1, 2]), &sv(&["a"])),
        Ok(list(vec![l(1), l(2), sym("a")]))
    );
    assert_eq!(join(&fv(&[1.5]), &l(1)), Ok(list(vec![f(1.5), l(1)])));
}

#[test]
fn join_symbols() {
    assert_eq!(join(&sym("a"), &sym("b")), Ok(sv(&["a", "b"])));
    assert_eq!(join(&sv(&["a", "b"]), &sym("c")), Ok(sv(&["a", "b", "c"])));
    assert_eq!(join(&sym("a"), &sv(&["b"])), Ok(sv(&["a", "b"])));
    assert_eq!(join(&sym(""), &sym("")), Ok(sv(&["", ""])));
    assert_eq!(join(&sym("a"), &l(1)), Ok(list(vec![sym("a"), l(1)])));
}

#[test]
fn join_strings() {
    assert_eq!(join(&cv("ab"), &cv("c")), Ok(cv("abc")));
    assert_eq!(join(&c('a'), &c('b')), Ok(cv("ab")));
    assert_eq!(join(&c('a'), &c(' ')), Ok(cv("a ")));
    assert_eq!(join(&cv("ab"), &c('c')), Ok(cv("abc")));
    assert_eq!(join(&c('a'), &cv("bc")), Ok(cv("abc")));
    assert_eq!(join(&c('a'), &l(1)), Ok(list(vec![c('a'), l(1)])));
}

#[test]
fn join_general_lists() {
    let g = list(vec![l(1), sym("a")]);
    assert_eq!(join(&g, &l(2)), Ok(list(vec![l(1), sym("a"), l(2)])));
    assert_eq!(join(&l(1), &g), Ok(list(vec![l(1), l(1), sym("a")])));
    assert_eq!(
        join(&g, &lv(&[1, 2])),
        Ok(list(vec![l(1), sym("a"), l(1), l(2)]))
    );
    assert_eq!(
        join(&lv(&[1, 2]), &list(vec![l(3), sym("a")])),
        Ok(list(vec![l(1), l(2), l(3), sym("a")]))
    );
    assert_eq!(
        join(&g, &list(vec![l(2), f(3.0)])),
        Ok(list(vec![l(1), sym("a"), l(2), f(3.0)]))
    );
    // a vector item stays one item
    assert_eq!(
        join(&list(vec![lv(&[1, 2]), l(3)]), &l(4)),
        Ok(list(vec![lv(&[1, 2]), l(3), l(4)]))
    );
    assert_eq!(
        join(&l(0), &list(vec![lv(&[1, 2])])),
        Ok(list(vec![l(0), lv(&[1, 2])]))
    );
}

#[test]
fn join_general_lists_normalise() {
    // (1;2),3 is simple in q; so is a list whose items end up one type
    assert_eq!(join(&list(vec![l(1), l(2)]), &l(3)), Ok(lv(&[1, 2, 3])));
    assert_eq!(join(&list(vec![l(1)]), &lv(&[2])), Ok(lv(&[1, 2])));
    assert_eq!(join(&list(vec![sym("a")]), &sym("b")), Ok(sv(&["a", "b"])));
}

#[test]
fn join_empty_operands() {
    // an empty vector or () is an identity; an atom becomes a one-item list
    assert_eq!(join(&lv(&[]), &l(1)), Ok(lv(&[1])));
    assert_eq!(join(&list(vec![]), &l(1)), Ok(lv(&[1])));
    assert_eq!(join(&l(1), &list(vec![])), Ok(lv(&[1])));
    assert_eq!(join(&l(1), &lv(&[])), Ok(lv(&[1])));
    assert_eq!(join(&list(vec![]), &c(' ')), Ok(cv(" ")));
    assert_eq!(join(&list(vec![]), &sym("a")), Ok(sv(&["a"])));
    assert_eq!(join(&list(vec![]), &lv(&[1, 2])), Ok(lv(&[1, 2])));
    assert_eq!(join(&lv(&[1, 2]), &list(vec![])), Ok(lv(&[1, 2])));
    assert_eq!(join(&list(vec![]), &list(vec![])), Ok(list(vec![])));
}

#[test]
fn join_empty_typed_vector_takes_the_other_type() {
    // q: (0#0),1.5 is ,1.5 and (0#0.0),1 is ,1
    assert_eq!(join(&lv(&[]), &f(1.5)), Ok(fv(&[1.5])));
    assert_eq!(join(&fv(&[]), &l(1)), Ok(lv(&[1])));
    assert_eq!(join(&lv(&[]), &sym("a")), Ok(sv(&["a"])));
    assert_eq!(join(&bv(&[]), &l(1)), Ok(lv(&[1])));
    assert_eq!(join(&lv(&[]), &b(true)), Ok(bv(&[true])));
    assert_eq!(join(&lv(&[]), &cv("ab")), Ok(cv("ab")));
    assert_eq!(join(&lv(&[1, 2]), &fv(&[])), Ok(lv(&[1, 2])));
    assert_eq!(join(&lv(&[1, 2]), &sv(&[])), Ok(lv(&[1, 2])));
    assert_eq!(join(&f(1.5), &lv(&[])), Ok(fv(&[1.5])));
    assert_eq!(
        join(&lv(&[]), &list(vec![l(1), sym("a")])),
        Ok(list(vec![l(1), sym("a")]))
    );
    assert_eq!(
        join(&list(vec![l(1), sym("a")]), &lv(&[])),
        Ok(list(vec![l(1), sym("a")]))
    );
    // both empty: the right type, unless the right is ()
    assert_eq!(join(&lv(&[]), &fv(&[])), Ok(fv(&[])));
    assert_eq!(join(&fv(&[]), &lv(&[])), Ok(lv(&[])));
    assert_eq!(join(&lv(&[]), &list(vec![])), Ok(lv(&[])));
    assert_eq!(join(&list(vec![]), &lv(&[])), Ok(lv(&[])));
    assert_eq!(join(&lv(&[]), &sv(&[])), Ok(sv(&[])));
}

#[test]
fn join_temporals_make_general_lists() {
    // ponytail: temporals have no column type
    let d = Value::Atom(Atom::NullDate);
    assert_eq!(join(&d, &d), Ok(list(vec![d.clone(), d.clone()])));
    assert_eq!(join(&d, &l(1)), Ok(list(vec![d.clone(), l(1)])));
    assert_eq!(join(&lv(&[]), &d), Ok(list(vec![d.clone()])));
    assert_eq!(join(&list(vec![]), &d), Ok(list(vec![d.clone()])));
}

#[test]
fn join_beyond_cap_is_domain() {
    let big = Value::Vector(Rc::new(Column::Long(vec![0; MAX_ELEMS])));
    assert_eq!(join(&big, &l(1)), Err(QError::Domain));
    assert_eq!(join(&l(1), &big), Err(QError::Domain));
    assert_eq!(join(&big, &big), Err(QError::Domain));
    assert_eq!(join(&big, &f(1.0)), Err(QError::Domain));
    // exactly at the cap is fine
    let almost = Value::Vector(Rc::new(Column::Long(vec![0; MAX_ELEMS - 1])));
    match join(&almost, &l(1)) {
        Ok(Value::Vector(c)) => assert_eq!(c.len(), MAX_ELEMS),
        other => panic!("{other:?}"),
    }
}

fn items(v: &Value) -> Vec<Value> {
    match v {
        Value::Atom(_) => vec![v.clone()],
        Value::Vector(c) => (0..c.len()).map(|i| Value::Atom(c.get(i))).collect(),
        Value::List(l) => l.to_vec(),
    }
}

fn arb_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        (-5i64..5).prop_map(l),
        (-5i64..5).prop_map(|n| f(n as f64)),
        proptest::sample::select(vec!["a", "b"]).prop_map(sym),
        proptest::collection::vec(-5i64..5, 0..4).prop_map(|v| lv(&v)),
        proptest::collection::vec(-5i64..5, 0..4)
            .prop_map(|v| fv(&v.iter().map(|&n| n as f64).collect::<Vec<_>>())),
        "[ab]{0,3}".prop_map(|s| cv(&s)),
        Just(list(vec![])),
        Just(list(vec![l(1), sym("a")])),
        Just(list(vec![lv(&[1, 2]), f(2.5)])),
    ]
}

proptest! {
    #[test]
    fn join_prop_length_adds(x in arb_value(), y in arb_value()) {
        let r = join(&x, &y).unwrap();
        prop_assert_eq!(items(&r).len(), items(&x).len() + items(&y).len());
    }

    #[test]
    fn join_prop_items_are_the_concatenation(x in arb_value(), y in arb_value()) {
        let want = Value::from_items([items(&x), items(&y)].concat());
        let got = join(&x, &y).unwrap();
        // from_items is the normal form; only the type of an empty result may differ
        if items(&got).is_empty() {
            prop_assert!(items(&want).is_empty());
        } else {
            prop_assert_eq!(got, want);
        }
    }

    #[test]
    fn join_prop_associative_same_type(
        a in proptest::collection::vec(-5i64..5, 0..5),
        b in proptest::collection::vec(-5i64..5, 0..5),
        c in proptest::collection::vec(-5i64..5, 0..5),
    ) {
        let want: Vec<i64> = [a.clone(), b.clone(), c.clone()].concat();
        let (a, b, c) = (lv(&a), lv(&b), lv(&c));
        let left = join(&join(&a, &b).unwrap(), &c);
        let right = join(&a, &join(&b, &c).unwrap());
        prop_assert_eq!(&left, &right);
        prop_assert_eq!(left, Ok(lv(&want)));
    }

    #[test]
    fn join_prop_empty_is_identity(v in proptest::collection::vec(-5i64..5, 0..6)) {
        prop_assert_eq!(join(&lv(&v), &lv(&[])), Ok(lv(&v)));
        prop_assert_eq!(join(&lv(&[]), &lv(&v)), Ok(lv(&v)));
        prop_assert_eq!(join(&lv(&v), &list(vec![])), Ok(lv(&v)));
        prop_assert_eq!(join(&list(vec![]), &lv(&v)), Ok(lv(&v)));
    }
}

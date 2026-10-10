//! `n#x` through the kernel. Every expected value was checked against q 4.1.
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
fn f(x: f64) -> Value {
    Value::Atom(Atom::Float(x))
}
fn take(n: &Value, x: &Value) -> Result<Value, QError> {
    dyad(Verb::Take, n, x)
}

#[test]
fn take_prefix() {
    assert_eq!(take(&l(2), &lv(&[1, 2, 3])), Ok(lv(&[1, 2])));
    assert_eq!(take(&l(3), &lv(&[1, 2, 3])), Ok(lv(&[1, 2, 3])));
    assert_eq!(take(&l(1), &cv("abc")), Ok(cv("a")));
    assert_eq!(take(&l(2), &sv(&["a", "b", "c"])), Ok(sv(&["a", "b"])));
    assert_eq!(take(&l(2), &fv(&[1.5, 2.5, 3.5])), Ok(fv(&[1.5, 2.5])));
    assert_eq!(take(&l(1), &bv(&[true, false])), Ok(bv(&[true])));
}

#[test]
fn take_wraps() {
    assert_eq!(take(&l(5), &lv(&[1, 2])), Ok(lv(&[1, 2, 1, 2, 1])));
    assert_eq!(take(&l(4), &cv("ab")), Ok(cv("abab")));
}

#[test]
fn take_negative_from_end() {
    assert_eq!(take(&l(-2), &lv(&[1, 2, 3])), Ok(lv(&[2, 3])));
    assert_eq!(take(&l(-5), &lv(&[1, 2, 3])), Ok(lv(&[2, 3, 1, 2, 3])));
    assert_eq!(take(&l(-1), &cv("abc")), Ok(cv("c")));
    assert_eq!(take(&l(-3), &lv(&[1, 2, 3])), Ok(lv(&[1, 2, 3])));
}

#[test]
fn take_from_atom() {
    assert_eq!(take(&l(3), &l(7)), Ok(lv(&[7, 7, 7])));
    assert_eq!(take(&l(-3), &l(7)), Ok(lv(&[7, 7, 7])));
    assert_eq!(
        take(&l(3), &Value::Atom(Atom::Character('a'))),
        Ok(cv("aaa"))
    );
    assert_eq!(take(&l(2), &sym("a")), Ok(sv(&["a", "a"])));
    assert_eq!(take(&l(3), &f(1.5)), Ok(fv(&[1.5, 1.5, 1.5])));
    assert_eq!(
        take(&l(2), &Value::Atom(Atom::Boolean(true))),
        Ok(bv(&[true, true]))
    );
    assert_eq!(take(&l(0), &l(7)), Ok(lv(&[])));
}

#[test]
fn take_zero_keeps_type() {
    assert_eq!(take(&l(0), &lv(&[1, 2])), Ok(lv(&[])));
    assert_eq!(take(&l(0), &cv("ab")), Ok(cv("")));
    assert_eq!(take(&l(0), &sv(&["a"])), Ok(sv(&[])));
    assert_eq!(take(&l(0), &fv(&[1.0])), Ok(fv(&[])));
    assert_eq!(take(&l(0), &bv(&[true])), Ok(bv(&[])));
    assert_eq!(take(&l(0), &list(vec![l(1), sym("a")])), Ok(list(vec![])));
}

#[test]
fn take_from_empty() {
    // typed nulls: 3#0#0 is 0N 0N 0N
    let nul = i64::MIN;
    assert_eq!(take(&l(3), &lv(&[])), Ok(lv(&[nul, nul, nul])));
    assert_eq!(take(&l(-2), &lv(&[])), Ok(lv(&[nul, nul])));
    assert_eq!(take(&l(2), &fv(&[])), Ok(fv(&[f64::NAN, f64::NAN])));
    assert_eq!(take(&l(3), &cv("")), Ok(cv("   ")));
    assert_eq!(take(&l(2), &sv(&[])), Ok(sv(&["", ""])));
    assert_eq!(take(&l(3), &bv(&[])), Ok(bv(&[false, false, false])));
    // the null of a general list is (): 2#() is (();())
    assert_eq!(
        take(&l(2), &list(vec![])),
        Ok(list(vec![list(vec![]), list(vec![])]))
    );
    assert_eq!(take(&l(0), &list(vec![])), Ok(list(vec![])));
    assert_eq!(take(&l(-1), &list(vec![])), Ok(list(vec![list(vec![])])));
}

#[test]
fn take_general_list() {
    let g = list(vec![l(1), f(2.5)]);
    assert_eq!(take(&l(2), &g), Ok(g.clone()));
    assert_eq!(take(&l(3), &g), Ok(list(vec![l(1), f(2.5), l(1)])));
    let m = list(vec![l(1), f(2.5), sym("a")]);
    assert_eq!(take(&l(-2), &m), Ok(list(vec![f(2.5), sym("a")])));
    assert_eq!(
        take(&l(5), &list(vec![l(1), sym("a")])),
        Ok(list(vec![l(1), sym("a"), l(1), sym("a"), l(1)]))
    );
    assert_eq!(
        take(&l(-3), &list(vec![l(1), sym("a")])),
        Ok(list(vec![sym("a"), l(1), sym("a")]))
    );
    // nested items are kept whole
    let n = list(vec![lv(&[1, 2]), l(3)]);
    assert_eq!(
        take(&l(3), &n),
        Ok(list(vec![lv(&[1, 2]), l(3), lv(&[1, 2])]))
    );
}

#[test]
fn take_general_list_normalises() {
    // the result is a simple list as soon as its items are all one type
    let g = list(vec![l(1), sym("a")]);
    assert_eq!(take(&l(1), &g), Ok(lv(&[1])));
    assert_eq!(take(&l(-1), &g), Ok(sv(&["a"])));
    assert_eq!(take(&l(0), &g), Ok(list(vec![])));
}

#[test]
fn take_non_long_count_is_type() {
    for bad in [
        f(2.0),
        f(2.5),
        Value::Atom(Atom::Character('a')),
        sym("a"),
        Value::Atom(Atom::NullDate),
        l(i64::MIN), // 0N#x is 'type in q
    ] {
        assert_eq!(take(&bad, &lv(&[1, 2, 3])), Err(QError::Type), "{bad:?}");
    }
}

#[test]
fn take_boolean_count_is_a_long() {
    // q: 1b#1 2 3 is ,1 and 0b#1 2 3 is empty
    assert_eq!(
        take(&Value::Atom(Atom::Boolean(true)), &lv(&[1, 2, 3])),
        Ok(lv(&[1]))
    );
    assert_eq!(
        take(&Value::Atom(Atom::Boolean(false)), &lv(&[1, 2, 3])),
        Ok(lv(&[]))
    );
}

#[test]
fn take_list_count_is_reshape_nyi() {
    assert_eq!(
        take(&lv(&[2, 3]), &lv(&[1, 2, 3, 4, 5, 6])),
        Err(QError::Nyi("reshape".into()))
    );
    assert_eq!(
        take(&list(vec![l(2), f(1.5)]), &lv(&[1])),
        Err(QError::Nyi("reshape".into()))
    );
}

#[test]
fn take_beyond_cap_is_domain() {
    let big = i64::try_from(MAX_ELEMS).unwrap() + 1;
    for n in [big, -big, i64::MAX, i64::MAX - 1] {
        assert_eq!(take(&l(n), &lv(&[1, 2])), Err(QError::Domain), "{n}");
        assert_eq!(take(&l(n), &l(1)), Err(QError::Domain), "{n}");
        assert_eq!(
            take(&l(n), &list(vec![l(1), sym("a")])),
            Err(QError::Domain),
            "{n}"
        );
        assert_eq!(take(&l(n), &lv(&[])), Err(QError::Domain), "{n}");
        assert_eq!(take(&l(n), &list(vec![])), Err(QError::Domain), "{n}");
        assert_eq!(
            take(&l(n), &Value::Atom(Atom::NullDate)),
            Err(QError::Domain),
            "{n}"
        );
    }
    // the cap itself is allowed
    let n = i64::try_from(MAX_ELEMS).unwrap();
    match take(&l(n), &l(1)) {
        Ok(Value::Vector(c)) => assert_eq!(c.len(), MAX_ELEMS),
        other => panic!("{other:?}"),
    }
}

#[test]
fn take_temporal_atom_is_a_general_list() {
    // ponytail: temporals have no column type, so the result is a general list
    let d = Value::Atom(Atom::NullDate);
    assert_eq!(take(&l(2), &d), Ok(list(vec![d.clone(), d.clone()])));
    assert_eq!(take(&l(0), &d), Ok(list(vec![])));
}

fn model(n: i64, v: &[i64]) -> Vec<i64> {
    let count = n.unsigned_abs() as usize;
    if v.is_empty() {
        return vec![i64::MIN; count];
    }
    let start = if n < 0 {
        (v.len() - count % v.len()) % v.len()
    } else {
        0
    };
    (0..count).map(|k| v[(start + k) % v.len()]).collect()
}

proptest! {
    #[test]
    fn take_prop_count(n in -300i64..300, v in proptest::collection::vec(-5i64..5, 0..8)) {
        let Ok(Value::Vector(c)) = take(&l(n), &lv(&v)) else { panic!() };
        prop_assert_eq!(c.len(), n.unsigned_abs() as usize);
    }

    #[test]
    fn take_prop_matches_model(n in -40i64..40, v in proptest::collection::vec(-5i64..5, 0..8)) {
        prop_assert_eq!(take(&l(n), &lv(&v)), Ok(lv(&model(n, &v))));
    }

    #[test]
    fn take_prop_full_length_is_identity(v in proptest::collection::vec(-5i64..5, 0..12)) {
        let n = v.len() as i64;
        prop_assert_eq!(take(&l(n), &lv(&v)), Ok(lv(&v)));
        prop_assert_eq!(take(&l(-n), &lv(&v)), Ok(lv(&v)));
    }

    #[test]
    fn take_prop_general_list_matches_model(n in -20i64..20, k in 2usize..5) {
        // items alternate long/symbol so the list stays general
        let items: Vec<Value> = (0..k).map(|i| if i % 2 == 0 { l(i as i64) } else { sym("a") }).collect();
        let g = Value::from_items(items.clone());
        let count = n.unsigned_abs() as usize;
        let start = if n < 0 { (k - count % k) % k } else { 0 };
        let want = Value::from_items((0..count).map(|j| items[(start + j) % k].clone()).collect());
        prop_assert_eq!(take(&l(n), &g), Ok(want));
    }
}

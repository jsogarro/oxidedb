use oxidedb::language::builtins::{call, lookup, Builtin};
use oxidedb::types::column::Column;
use oxidedb::types::sym::Sym;
use oxidedb::{Atom, QError, Value};
use proptest::prelude::*;
use std::rc::Rc;

fn int(n: i64) -> Value {
    Value::Atom(Atom::Integer(n))
}

fn longs(v: &[i64]) -> Value {
    Value::Vector(Rc::new(Column::Long(v.to_vec())))
}

fn til(n: i64) -> Result<Value, QError> {
    call("til", &[int(n)])
}

#[test]
fn builtin_til() {
    assert_eq!(til(5).unwrap(), longs(&[0, 1, 2, 3, 4]));
    assert_eq!(til(1).unwrap(), longs(&[0]));
    assert_eq!(til(0).unwrap(), longs(&[]));
    assert_eq!(til(0).unwrap().type_code(), 7);
}

#[test]
fn builtin_til_negative_is_domain() {
    assert_eq!(til(-1), Err(QError::Domain));
    assert_eq!(til(i64::MIN), Err(QError::Domain));
}

#[test]
fn builtin_til_huge_is_domain_without_allocating() {
    assert_eq!(til(100_000_001), Err(QError::Domain));
    assert_eq!(til(i64::MAX), Err(QError::Domain));
    assert_eq!(til(100_000_000).unwrap().type_code(), 7);
}

#[test]
fn builtin_til_non_long_is_type() {
    for v in [
        Value::Atom(Atom::Float(3.0)),
        Value::Atom(Atom::Boolean(true)),
        Value::Atom(Atom::Character('a')),
        Value::Atom(Atom::Symbol(Sym::intern("a"))),
        longs(&[1, 2]),
        Value::List(Rc::new(vec![int(1), Value::Atom(Atom::Float(1.0))])),
    ] {
        assert_eq!(call("til", &[v]), Err(QError::Type));
    }
}

#[test]
fn builtin_count() {
    assert_eq!(call("count", &[longs(&[7, 8, 9])]).unwrap(), int(3));
    assert_eq!(call("count", &[longs(&[])]).unwrap(), int(0));
    assert_eq!(call("count", &[int(5)]).unwrap(), int(1));
    assert_eq!(
        call("count", &[Value::Atom(Atom::Symbol(Sym::intern("a")))]).unwrap(),
        int(1)
    );
    let general = Value::List(Rc::new(vec![
        int(1),
        Value::Atom(Atom::Float(1.0)),
        longs(&[1, 2, 3]),
    ]));
    assert_eq!(call("count", &[general]).unwrap(), int(3));
    assert_eq!(
        call("count", &[Value::List(Rc::new(vec![]))]).unwrap(),
        int(0)
    );
    let chars = Value::Vector(Rc::new(Column::Char(vec!['a', 'b'])));
    assert_eq!(call("count", &[chars]).unwrap(), int(2));
}

#[test]
fn builtin_lookup_unknown_is_none() {
    assert_eq!(lookup("nope"), None);
    assert_eq!(lookup(""), None);
    assert_eq!(lookup("Til"), None);
    assert_eq!(lookup("til"), Some(Builtin::Til));
    assert_eq!(lookup("count"), Some(Builtin::Count));
    assert!(matches!(call("nope", &[int(1)]), Err(QError::Undefined(n)) if n == "nope"));
}

#[test]
fn builtin_arity_is_rank() {
    for name in ["til", "count"] {
        assert_eq!(call(name, &[]), Err(QError::Rank));
        assert_eq!(call(name, &[int(1), int(2)]), Err(QError::Rank));
    }
}

proptest! {
    #[test]
    fn builtin_count_til_is_n(n in 0i64..10_000) {
        let t = til(n).unwrap();
        prop_assert_eq!(call("count", &[t]).unwrap(), int(n));
    }

    #[test]
    fn builtin_til_is_sorted_from_zero(n in 1i64..10_000) {
        let Value::Vector(c) = til(n).unwrap() else { panic!("not a vector") };
        let Column::Long(v) = &*c else { panic!("not long") };
        prop_assert_eq!(v[0], 0);
        prop_assert!(v.windows(2).all(|w| w[1] == w[0] + 1));
    }
}

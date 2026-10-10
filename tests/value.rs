use oxidedb::language::ast::{Expr, Verb};
use oxidedb::types::column::Column;
use oxidedb::types::sym::Sym;
use oxidedb::{Atom, Interpreter, QError, Value};
use std::rc::Rc;

fn int(n: i64) -> Value {
    Value::Atom(Atom::Integer(n))
}

fn longs(v: &[i64]) -> Value {
    Value::Vector(Rc::new(Column::Long(v.to_vec())))
}

fn floats(v: &[f64]) -> Value {
    Value::Vector(Rc::new(Column::Float(v.to_vec())))
}

#[test]
fn value_eval_returns_value() {
    let mut i = Interpreter::new();
    assert_eq!(i.eval_line("2+3").unwrap(), Some(int(5)));
    assert_eq!(i.eval_line("x:7").unwrap(), Some(int(7)));
    assert_eq!(i.eval_line("x*2").unwrap(), Some(int(14)));
    assert_eq!(i.eval_line("// c").unwrap(), None);
}

#[test]
fn value_eq_atom_shim() {
    assert_eq!(int(5), Atom::Integer(5));
    assert_eq!(Atom::Integer(5), int(5));
    assert_ne!(int(5), Atom::Integer(6));
    assert_ne!(Atom::Integer(6), int(5));
    // A non-atom value never equals an atom.
    assert_ne!(longs(&[5]), Atom::Integer(5));
    assert_ne!(Atom::Integer(5), longs(&[5]));
    assert_ne!(Value::List(Rc::new(vec![int(5)])), Atom::Integer(5));
}

fn atoms(a: &[Atom]) -> Vec<Value> {
    a.iter().cloned().map(Value::Atom).collect()
}

#[test]
fn value_from_items_collapses() {
    use Atom::*;
    assert_eq!(
        Value::from_items(atoms(&[Integer(1), Integer(2), Integer(3)])),
        longs(&[1, 2, 3])
    );
    assert_eq!(
        Value::from_items(atoms(&[Boolean(true), Boolean(false)])),
        Value::Vector(Rc::new(Column::Bool(vec![true, false])))
    );
    assert_eq!(
        Value::from_items(atoms(&[Float(1.5), Float(2.0)])),
        floats(&[1.5, 2.0])
    );
    assert_eq!(
        Value::from_items(atoms(&[Character('a'), Character('b')])),
        Value::Vector(Rc::new(Column::Char(vec!['a', 'b'])))
    );
    assert_eq!(
        Value::from_items(atoms(&[Atom::from("a"), Atom::from("b")])),
        Value::Vector(Rc::new(Column::Sym(vec![
            Sym::intern("a"),
            Sym::intern("b")
        ])))
    );
    // A single atom is still a one-item vector.
    assert_eq!(Value::from_items(atoms(&[Integer(1)])), longs(&[1]));
}

#[test]
fn value_from_items_keeps_mixed_and_empty_as_lists() {
    use Atom::*;
    // No int -> float promotion.
    let mixed = atoms(&[Integer(1), Float(2.0)]);
    assert_eq!(
        Value::from_items(mixed.clone()),
        Value::List(Rc::new(mixed))
    );
    // Empty input is the empty general list.
    assert_eq!(Value::from_items(vec![]), Value::List(Rc::new(vec![])));
    // Nested items stay a list.
    let nested = vec![longs(&[1, 2]), longs(&[3])];
    assert_eq!(
        Value::from_items(nested.clone()),
        Value::List(Rc::new(nested))
    );
    let inner = vec![Value::List(Rc::new(vec![]))];
    assert_eq!(
        Value::from_items(inner.clone()),
        Value::List(Rc::new(inner))
    );
    // Temporal atoms have no column type.
    let dates = vec![Value::Atom(NullDate), Value::Atom(NullDate)];
    assert_eq!(
        Value::from_items(dates.clone()),
        Value::List(Rc::new(dates))
    );
}

#[test]
fn value_type_codes() {
    assert_eq!(int(1).type_code(), -7);
    assert_eq!(Value::Atom(Atom::Float(1.0)).type_code(), -9);
    assert_eq!(longs(&[1]).type_code(), 7);
    assert_eq!(floats(&[1.0]).type_code(), 9);
    assert_eq!(longs(&[]).type_code(), 7);
    assert_eq!(Value::List(Rc::new(vec![])).type_code(), 0);
    assert_eq!(
        Value::List(Rc::new(vec![int(1), Value::Atom(Atom::Float(1.0))])).type_code(),
        0
    );
}

#[test]
fn value_display() {
    assert_eq!(int(5).to_string(), "5");
    assert_eq!(Value::Atom(Atom::Float(f64::NAN)).to_string(), "0n");
    assert_eq!(longs(&[1, 2, 3]).to_string(), "1 2 3");
    assert_eq!(longs(&[1]).to_string(), ",1");
    assert_eq!(Value::List(Rc::new(vec![])).to_string(), "()");
    let l = Value::List(Rc::new(vec![
        int(1),
        Value::Atom(Atom::Float(2.5)),
        longs(&[3, 4]),
    ]));
    assert_eq!(l.to_string(), "1\n2.5\n3 4");
}

#[test]
fn value_null_equality() {
    let long_null = int(i64::MIN);
    assert_eq!(long_null, long_null.clone());
    let float_null = Value::Atom(Atom::Float(f64::NAN));
    assert_eq!(float_null, float_null.clone());
    assert_eq!(float_null, Atom::Float(f64::NAN));
    assert_ne!(float_null, Value::Atom(Atom::Float(1.0)));
    assert_ne!(Value::Atom(Atom::Float(1.0)), float_null);
    // Vectors and lists holding nulls.
    let v = floats(&[1.0, f64::NAN]);
    assert_eq!(v, v.clone());
    assert_ne!(v, floats(&[1.0, 2.0]));
    assert_ne!(v, floats(&[1.0]));
    assert_eq!(longs(&[1, i64::MIN]), longs(&[1, i64::MIN]));
    let l = Value::List(Rc::new(vec![float_null.clone(), v.clone()]));
    assert_eq!(l, l.clone());
    // 0f and -0f stay equal.
    assert_eq!(
        Value::Atom(Atom::Float(0.0)),
        Value::Atom(Atom::Float(-0.0))
    );
    assert_eq!(floats(&[0.0]), floats(&[-0.0]));
    // Shapes never compare equal across variants.
    assert_ne!(longs(&[1]), Value::List(Rc::new(vec![int(1)])));
    assert_ne!(
        Value::List(Rc::new(vec![int(1)])),
        Value::List(Rc::new(vec![int(2)]))
    );
    assert_ne!(longs(&[1]), floats(&[1.0]));
}

#[test]
fn value_non_atom_arithmetic_evaluates() {
    let mut i = Interpreter::new();
    i.set("v", longs(&[1, 2, 3]));
    i.set("l", Value::List(Rc::new(vec![int(1)])));
    for (src, want) in [
        ("v+1", longs(&[2, 3, 4])),
        ("1+v", longs(&[2, 3, 4])),
        ("v*v", longs(&[1, 4, 9])),
        ("l-1", longs(&[0])),
        ("1%l", floats(&[1.0])),
        ("-v", longs(&[-1, -2, -3])),
        ("-l", longs(&[-1])),
    ] {
        assert_eq!(i.eval_line(src), Ok(Some(want)), "{src}");
    }
    // Bound values are still readable.
    assert_eq!(i.get("v"), Some(&longs(&[1, 2, 3])));
    assert_eq!(i.get("nope"), None);
    assert_eq!(i.eval_line("v").unwrap(), Some(longs(&[1, 2, 3])));
}

#[test]
fn value_unimplemented_verbs_are_nyi() {
    let (operator, sym) = (Verb::Key, "!");
    let expr = Expr::BinaryOp {
        left: Box::new(Expr::Lit(Value::Atom(Atom::Integer(1)))),
        operator,
        right: Box::new(Expr::Lit(Value::Atom(Atom::Integer(2)))),
    };
    assert_eq!(
        Interpreter::new().evaluate(expr),
        Err(QError::Nyi(sym.into()))
    );
    // Mixed int/float must not fall into float promotion.
    let expr = Expr::BinaryOp {
        left: Box::new(Expr::Lit(Value::Atom(Atom::Integer(1)))),
        operator,
        right: Box::new(Expr::Lit(Value::Atom(Atom::Float(2.0)))),
    };
    assert_eq!(
        Interpreter::new().evaluate(expr),
        Err(QError::Nyi(sym.into()))
    );
}

#[test]
fn column_equality_is_per_type() {
    let s = Sym::intern;
    let pairs = [
        (Column::Bool(vec![true]), Column::Bool(vec![false])),
        (Column::Long(vec![1]), Column::Long(vec![2])),
        (Column::Float(vec![1.0]), Column::Float(vec![2.0])),
        (Column::Char(vec!['a']), Column::Char(vec!['b'])),
        (Column::Sym(vec![s("a")]), Column::Sym(vec![s("b")])),
    ];
    for (a, b) in pairs {
        assert_eq!(a, a.clone());
        assert_ne!(a, b);
    }
    assert_ne!(Column::Long(vec![]), Column::Float(vec![]));
}

fn atom_samples() -> Vec<(Atom, Atom)> {
    use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
    let d = |day| NaiveDate::from_ymd_opt(2024, 1, day).unwrap();
    let t = |s| NaiveTime::from_hms_opt(9, 30, s).unwrap();
    let ts = |s| Utc.with_ymd_and_hms(2024, 1, 15, 9, 30, s).unwrap();
    vec![
        (Atom::Boolean(true), Atom::Boolean(false)),
        (Atom::Integer(1), Atom::Integer(2)),
        (Atom::Float(1.5), Atom::Float(2.5)),
        (Atom::Character('a'), Atom::Character('b')),
        (Atom::from("a"), Atom::from("b")),
        (Atom::Date(d(1)), Atom::Date(d(2))),
        (Atom::Time(t(1)), Atom::Time(t(2))),
        (Atom::Timestamp(ts(1)), Atom::Timestamp(ts(2))),
        (Atom::NullDate, Atom::NullDate),
        (Atom::NullTime, Atom::NullTime),
        (Atom::NullTimestamp, Atom::NullTimestamp),
    ]
}

#[test]
fn atom_equality_table() {
    let s = atom_samples();
    for (i, (a, b)) in s.iter().enumerate() {
        assert_eq!(a, &a.clone(), "{a:?} reflexive");
        assert_eq!(b, &b.clone(), "{b:?} reflexive");
        if !matches!(a, Atom::NullDate | Atom::NullTime | Atom::NullTimestamp) {
            assert_ne!(a, b, "{a:?} vs {b:?}");
            assert_ne!(b, a);
        }
        for (j, (c, d)) in s.iter().enumerate() {
            if i != j {
                assert_ne!(a, c, "{a:?} vs {c:?}");
                assert_ne!(a, d, "{a:?} vs {d:?}");
                assert_ne!(b, c, "{b:?} vs {c:?}");
            }
        }
    }
    let f = Atom::Float;
    assert_eq!(f(f64::INFINITY), f(f64::INFINITY));
    assert_ne!(f(f64::INFINITY), f(f64::NEG_INFINITY));
    assert_ne!(f(f64::NEG_INFINITY), f(f64::INFINITY));
    assert_eq!(f(f64::NAN), f(f64::NAN));
    assert_eq!(f(0.0), f(-0.0));
    assert_ne!(Atom::Integer(1), f(1.0));
    assert_ne!(f(1.0), Atom::Integer(1));
    assert_ne!(Atom::Boolean(true), Atom::Integer(1));
    assert_ne!(Atom::Integer(i64::MIN), f(f64::NAN));
}

#[test]
fn column_equality_checks_every_element_and_length() {
    let s = Sym::intern;
    let cols = [
        (
            Column::Bool(vec![true, true, true]),
            Column::Bool(vec![true, true, false]),
        ),
        (Column::Long(vec![1, 2, 3]), Column::Long(vec![1, 2, 4])),
        (
            Column::Float(vec![1.0, 2.0, 3.0]),
            Column::Float(vec![1.0, 2.0, 4.0]),
        ),
        (
            Column::Char(vec!['a', 'b', 'c']),
            Column::Char(vec!['a', 'b', 'd']),
        ),
        (
            Column::Sym(vec![s("a"), s("b"), s("c")]),
            Column::Sym(vec![s("a"), s("b"), s("d")]),
        ),
    ];
    for (a, b) in cols {
        assert_eq!(a, a.clone());
        assert_ne!(a, b);
        assert_ne!(b, a);
        // Same prefix, different length.
        let short = Column::from_atoms(&(0..2).map(|i| a.get(i)).collect::<Vec<_>>()).unwrap();
        assert_ne!(a, short);
        assert_ne!(short, a);
    }
}

#[test]
fn value_non_atom_assignment_and_sharing() {
    let mut i = Interpreter::new();
    let col = Rc::new(Column::Long(vec![1, 2, 3]));
    i.set("v", Value::Vector(col.clone()));
    // Assignment returns the vector, and both names read back equal.
    assert_eq!(i.eval_line("w:v").unwrap(), Some(longs(&[1, 2, 3])));
    assert_eq!(i.get("w"), i.get("v"));
    assert_eq!(i.eval_line("a:b:v").unwrap(), Some(longs(&[1, 2, 3])));
    assert_eq!(i.get("a"), Some(&longs(&[1, 2, 3])));
    assert_eq!(i.get("b"), Some(&longs(&[1, 2, 3])));
    // Reading and rebinding share the column instead of copying it.
    let ptr = |v: Option<&Value>| match v {
        Some(Value::Vector(c)) => c.clone(),
        other => panic!("not a vector: {other:?}"),
    };
    assert!(Rc::ptr_eq(&ptr(i.get("v")), &col));
    assert!(Rc::ptr_eq(&ptr(i.get("w")), &col));
    assert!(Rc::ptr_eq(&ptr(i.get("b")), &col));
    match i.eval_line("v").unwrap() {
        Some(Value::Vector(c)) => assert!(Rc::ptr_eq(&c, &col)),
        other => panic!("{other:?}"),
    }
    // set overwrites.
    i.set("v", int(9));
    assert_eq!(i.get("v"), Some(&int(9)));
    assert_eq!(i.eval_line("v+1").unwrap(), Some(int(10)));
}

#[test]
fn value_from_items_keeps_every_non_atom_item() {
    let a = int(1);
    let v = longs(&[2, 3]);
    let items = vec![a.clone(), v.clone()];
    assert_eq!(
        Value::from_items(items.clone()),
        Value::List(Rc::new(items))
    );
    let items = vec![v.clone()];
    assert_eq!(
        Value::from_items(items.clone()),
        Value::List(Rc::new(items))
    );
    let items = vec![v.clone(), a.clone(), v.clone()];
    match Value::from_items(items) {
        Value::List(l) => assert_eq!(*l, vec![v.clone(), a, v.clone()]),
        other => panic!("{other:?}"),
    }
    let inner = Value::List(Rc::new(vec![int(1), v.clone()]));
    let items = vec![inner.clone(), int(5)];
    match Value::from_items(items) {
        Value::List(l) => assert_eq!(*l, vec![inner, int(5)]),
        other => panic!("{other:?}"),
    }
}

#[test]
fn value_new_verbs_on_any_operand_are_nyi_by_verb() {
    let mut i = Interpreter::new();
    i.set("v", longs(&[1, 2]));
    for (l, r) in [
        (Atom::Boolean(true), Atom::Boolean(false)),
        (Atom::Character('a'), Atom::Character('b')),
    ] {
        let e = Expr::BinaryOp {
            left: Box::new(Expr::Lit(Value::Atom(l))),
            operator: Verb::Key,
            right: Box::new(Expr::Lit(Value::Atom(r))),
        };
        assert_eq!(i.evaluate(e), Err(QError::Nyi("!".into())));
    }
    let e = Expr::BinaryOp {
        left: Box::new(Expr::Symbol("v".into())),
        operator: Verb::Key,
        right: Box::new(Expr::Lit(Value::Atom(Atom::Integer(1)))),
    };
    assert_eq!(i.evaluate(e), Err(QError::Nyi("!".into())));
}

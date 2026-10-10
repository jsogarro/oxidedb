use oxidedb::types::column::Column;
use oxidedb::types::sym::Sym;
use oxidedb::Atom;

fn sym(s: &str) -> Sym {
    Sym::intern(s)
}

#[test]
fn vector_type_codes() {
    assert_eq!(Column::Bool(vec![true]).type_code(), 1);
    assert_eq!(Column::Long(vec![1]).type_code(), 7);
    assert_eq!(Column::Float(vec![1.0]).type_code(), 9);
    assert_eq!(Column::Char(vec!['a']).type_code(), 10);
    assert_eq!(Column::Sym(vec![sym("a")]).type_code(), 11);
    // empty columns keep their type
    assert_eq!(Column::Long(vec![]).type_code(), 7);
}

#[test]
fn vector_len_and_get() {
    let c = Column::Long(vec![10, 20, 30]);
    assert_eq!(c.len(), 3);
    assert!(!c.is_empty());
    assert!(Column::Float(vec![]).is_empty());
    assert_eq!(c.get(0), Atom::Integer(10));
    assert_eq!(c.get(2), Atom::Integer(30));
    assert_eq!(c.get(3), Atom::Integer(i64::MIN));
    assert_eq!(c.get(usize::MAX), Atom::Integer(i64::MIN));
}

#[test]
fn vector_get_every_type() {
    assert_eq!(Column::Bool(vec![true, false]).get(0), Atom::Boolean(true));
    assert_eq!(Column::Bool(vec![true, false]).get(1), Atom::Boolean(false));
    assert_eq!(Column::Float(vec![2.5]).get(0), Atom::Float(2.5));
    assert_eq!(Column::Char(vec!['x']).get(0), Atom::Character('x'));
    assert_eq!(Column::Sym(vec![sym("a")]).get(0), Atom::Symbol(sym("a")));
}

#[test]
fn vector_out_of_range_is_typed_null() {
    // q has no boolean null: out of range reads give 0b
    assert_eq!(Column::Bool(vec![true]).get(1), Atom::Boolean(false));
    assert!(matches!(Column::Float(vec![1.0]).get(5), Atom::Float(f) if f.is_nan()));
    assert_eq!(Column::Char(vec!['a']).get(5), Atom::Character(' '));
    assert_eq!(Column::Sym(vec![sym("a")]).get(5), Atom::Symbol(Sym::NULL));
    assert!(Column::Long(vec![]).get(0).is_null());
    assert!(Column::Float(vec![]).get(0).is_null());
    assert!(Column::Char(vec![]).get(0).is_null());
    assert!(Column::Sym(vec![]).get(0).is_null());
}

#[test]
fn vector_null_atom_per_type() {
    assert_eq!(Column::Bool(vec![]).null_atom(), Atom::Boolean(false));
    assert_eq!(Column::Long(vec![]).null_atom(), Atom::Integer(i64::MIN));
    assert!(matches!(Column::Float(vec![]).null_atom(), Atom::Float(f) if f.is_nan()));
    assert_eq!(Column::Char(vec![]).null_atom(), Atom::Character(' '));
    assert_eq!(Column::Sym(vec![]).null_atom(), Atom::Symbol(Sym::NULL));
}

#[test]
fn vector_from_atoms_homogeneous() {
    let long = [Atom::Integer(1), Atom::Integer(i64::MIN), Atom::Integer(3)];
    assert_eq!(
        Column::from_atoms(&long),
        Some(Column::Long(vec![1, i64::MIN, 3]))
    );
    assert_eq!(
        Column::from_atoms(&[Atom::Boolean(true), Atom::Boolean(false)]),
        Some(Column::Bool(vec![true, false]))
    );
    assert_eq!(
        Column::from_atoms(&[Atom::Float(1.5)]),
        Some(Column::Float(vec![1.5]))
    );
    assert_eq!(
        Column::from_atoms(&[Atom::Character('a'), Atom::Character('b')]),
        Some(Column::Char(vec!['a', 'b']))
    );
    assert_eq!(
        Column::from_atoms(&[Atom::from("a"), Atom::from("b")]),
        Some(Column::Sym(vec![sym("a"), sym("b")]))
    );
}

#[test]
fn vector_from_atoms_rejects_mixed() {
    assert_eq!(
        Column::from_atoms(&[Atom::Integer(1), Atom::Float(2.0)]),
        None
    );
    assert_eq!(
        Column::from_atoms(&[Atom::Float(2.0), Atom::Integer(1)]),
        None
    );
    assert_eq!(
        Column::from_atoms(&[Atom::Integer(1), Atom::Integer(2), Atom::Boolean(true)]),
        None
    );
    assert_eq!(
        Column::from_atoms(&[Atom::Character('a'), Atom::from("a")]),
        None
    );
}

#[test]
fn vector_from_atoms_rejects_empty_and_unsupported() {
    assert_eq!(Column::from_atoms(&[]), None);
    assert_eq!(Column::from_atoms(&[Atom::NullDate]), None);
}

#[test]
fn vector_from_atoms_round_trips_through_get() {
    let atoms = [Atom::Integer(4), Atom::Integer(5)];
    let c = Column::from_atoms(&atoms).unwrap();
    assert_eq!((0..c.len()).map(|i| c.get(i)).collect::<Vec<_>>(), atoms);
}

#[test]
fn vector_from_atoms_rejects_every_cross_type_pair() {
    let all = [
        Atom::Boolean(true),
        Atom::Integer(1),
        Atom::Float(1.0),
        Atom::Character('a'),
        Atom::from("a"),
    ];
    for (i, a) in all.iter().enumerate() {
        for (j, b) in all.iter().enumerate() {
            if i != j {
                assert_eq!(
                    Column::from_atoms(&[a.clone(), b.clone()]),
                    None,
                    "{a:?} {b:?}"
                );
                assert_eq!(
                    Column::from_atoms(&[a.clone(), a.clone(), b.clone()]),
                    None,
                    "{a:?} {a:?} {b:?}"
                );
            }
        }
    }
}

#[test]
fn vector_from_atoms_keeps_all_elements() {
    assert_eq!(
        Column::from_atoms(&[
            Atom::Boolean(true),
            Atom::Boolean(false),
            Atom::Boolean(true)
        ]),
        Some(Column::Bool(vec![true, false, true]))
    );
    assert_eq!(
        Column::from_atoms(&[Atom::Float(1.0), Atom::Float(2.0), Atom::Float(3.0)]),
        Some(Column::Float(vec![1.0, 2.0, 3.0]))
    );
    assert_eq!(
        Column::from_atoms(&[
            Atom::Character('a'),
            Atom::Character('b'),
            Atom::Character('c')
        ]),
        Some(Column::Char(vec!['a', 'b', 'c']))
    );
    assert_eq!(
        Column::from_atoms(&[Atom::from("a"), Atom::from("b"), Atom::from("c")]),
        Some(Column::Sym(vec![sym("a"), sym("b"), sym("c")]))
    );
    assert_eq!(
        Column::from_atoms(&[Atom::Integer(1), Atom::Integer(2), Atom::Integer(3)]),
        Some(Column::Long(vec![1, 2, 3]))
    );
}

#[test]
fn vector_get_at_len_is_null_for_every_type() {
    assert_eq!(Column::Bool(vec![true, true]).get(2), Atom::Boolean(false));
    assert_eq!(Column::Long(vec![1, 2]).get(2), Atom::Integer(i64::MIN));
    assert!(Column::Float(vec![1.0, 2.0]).get(2).is_null());
    assert_eq!(Column::Char(vec!['a', 'b']).get(2), Atom::Character(' '));
    assert_eq!(
        Column::Sym(vec![sym("a"), sym("b")]).get(2),
        Atom::Symbol(Sym::NULL)
    );
    assert_eq!(Column::Char(vec!['a', 'b']).get(1), Atom::Character('b'));
    assert_eq!(
        Column::Sym(vec![sym("a"), sym("b")]).get(1),
        Atom::Symbol(sym("b"))
    );
}

#[test]
fn vector_mixed_list_has_type_zero() {
    use oxidedb::Value;
    use std::rc::Rc;
    let mixed = vec![Value::Atom(Atom::Integer(1)), Value::Atom(Atom::Float(2.0))];
    // Mixed atoms never become a column; the result is a general list, type 0.
    assert!(Column::from_atoms(&[Atom::Integer(1), Atom::Float(2.0)]).is_none());
    let v = Value::from_items(mixed.clone());
    assert_eq!(v, Value::List(Rc::new(mixed)));
    assert_eq!(v.type_code(), 0);
}

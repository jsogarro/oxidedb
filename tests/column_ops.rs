use oxidedb::types::column::Column;
use oxidedb::types::sym::Sym;

fn s(n: &str) -> Sym {
    Sym::intern(n)
}

fn all_types() -> Vec<Column> {
    vec![
        Column::Bool(vec![true, false, true]),
        Column::Long(vec![10, 20, 30]),
        Column::Float(vec![1.5, 2.5, 3.5]),
        Column::Char(vec!['a', 'b', 'c']),
        Column::Sym(vec![s("a"), s("b"), s("c")]),
    ]
}

fn empty_like(c: &Column) -> Column {
    c.index(&[])
}

#[test]
fn index_gathers_in_order_with_repeats() {
    assert_eq!(
        Column::Long(vec![10, 20, 30]).index(&[2, 0, 0]),
        Column::Long(vec![30, 10, 10])
    );
    assert_eq!(
        Column::Bool(vec![true, false]).index(&[1, 0]),
        Column::Bool(vec![false, true])
    );
    assert_eq!(
        Column::Float(vec![1.5, 2.5]).index(&[1]),
        Column::Float(vec![2.5])
    );
    assert_eq!(
        Column::Char(vec!['a', 'b']).index(&[1, 0]),
        Column::Char(vec!['b', 'a'])
    );
    assert_eq!(
        Column::Sym(vec![s("a"), s("b")]).index(&[1]),
        Column::Sym(vec![s("b")])
    );
}

#[test]
fn index_out_of_range_and_negative_are_typed_nulls() {
    assert_eq!(
        Column::Long(vec![1, 2]).index(&[2, -1, i64::MIN, i64::MAX]),
        Column::Long(vec![i64::MIN; 4])
    );
    assert_eq!(
        Column::Bool(vec![true]).index(&[1, -1]),
        Column::Bool(vec![false, false])
    );
    assert_eq!(
        Column::Float(vec![1.0]).index(&[1, -1]),
        Column::Float(vec![f64::NAN, f64::NAN])
    );
    assert_eq!(
        Column::Char(vec!['a']).index(&[1, -1]),
        Column::Char(vec![' ', ' '])
    );
    assert_eq!(
        Column::Sym(vec![s("a")]).index(&[1, -1]),
        Column::Sym(vec![Sym::NULL, Sym::NULL])
    );
    // the last valid index is not null
    assert_eq!(Column::Long(vec![1, 2]).index(&[1]), Column::Long(vec![2]));
}

#[test]
fn index_empty_idx_keeps_type() {
    for c in all_types() {
        let e = c.index(&[]);
        assert!(e.is_empty());
        assert_eq!(e.type_code(), c.type_code());
    }
}

#[test]
fn take_first_n_and_wrap() {
    assert_eq!(
        Column::Long(vec![1, 2]).take(5),
        Column::Long(vec![1, 2, 1, 2, 1])
    );
    assert_eq!(
        Column::Long(vec![1, 2, 3]).take(2),
        Column::Long(vec![1, 2])
    );
    assert_eq!(
        Column::Long(vec![1, 2, 3]).take(3),
        Column::Long(vec![1, 2, 3])
    );
    assert_eq!(
        Column::Long(vec![1, 2, 3]).take(7),
        Column::Long(vec![1, 2, 3, 1, 2, 3, 1])
    );
}

#[test]
fn take_negative_takes_from_the_end_and_wraps() {
    assert_eq!(
        Column::Long(vec![1, 2, 3]).take(-2),
        Column::Long(vec![2, 3])
    );
    assert_eq!(
        Column::Long(vec![1, 2, 3]).take(-3),
        Column::Long(vec![1, 2, 3])
    );
    assert_eq!(
        Column::Long(vec![1, 2]).take(-5),
        Column::Long(vec![2, 1, 2, 1, 2])
    );
    assert_eq!(
        Column::Long(vec![1, 2, 3]).take(-4),
        Column::Long(vec![3, 1, 2, 3])
    );
    assert_eq!(Column::Long(vec![1, 2, 3]).take(-1), Column::Long(vec![3]));
}

#[test]
fn take_zero_is_empty_same_type() {
    for c in all_types() {
        let e = c.take(0);
        assert!(e.is_empty());
        assert_eq!(e.type_code(), c.type_code());
    }
}

#[test]
fn take_every_type() {
    assert_eq!(
        Column::Bool(vec![true, false]).take(3),
        Column::Bool(vec![true, false, true])
    );
    assert_eq!(
        Column::Float(vec![1.5, 2.5]).take(-3),
        Column::Float(vec![2.5, 1.5, 2.5])
    );
    assert_eq!(
        Column::Char(vec!['a', 'b']).take(3),
        Column::Char(vec!['a', 'b', 'a'])
    );
    assert_eq!(
        Column::Sym(vec![s("a"), s("b")]).take(-1),
        Column::Sym(vec![s("b")])
    );
}

// ponytail: verify against q — taking from an empty list is assumed to give typed nulls.
#[test]
fn take_from_empty_is_typed_nulls() {
    for c in all_types() {
        let e = empty_like(&c);
        for n in [3, -3] {
            let t = e.take(n);
            assert_eq!(t.type_code(), c.type_code());
            assert_eq!(t.len(), 3);
            assert_eq!(t, c.index(&[-1, -1, -1]));
        }
        assert!(e.take(0).is_empty());
    }
}

#[test]
fn concat_same_type_joins() {
    assert_eq!(
        Column::Long(vec![1]).concat(&Column::Long(vec![2, 3])),
        Some(Column::Long(vec![1, 2, 3]))
    );
    assert_eq!(
        Column::Bool(vec![true]).concat(&Column::Bool(vec![false])),
        Some(Column::Bool(vec![true, false]))
    );
    assert_eq!(
        Column::Float(vec![1.0]).concat(&Column::Float(vec![2.0])),
        Some(Column::Float(vec![1.0, 2.0]))
    );
    assert_eq!(
        Column::Char(vec!['a']).concat(&Column::Char(vec!['b'])),
        Some(Column::Char(vec!['a', 'b']))
    );
    assert_eq!(
        Column::Sym(vec![s("a")]).concat(&Column::Sym(vec![s("b")])),
        Some(Column::Sym(vec![s("a"), s("b")]))
    );
}

#[test]
fn concat_empty_sides() {
    let a = Column::Long(vec![1, 2]);
    assert_eq!(a.concat(&Column::Long(vec![])), Some(a.clone()));
    assert_eq!(Column::Long(vec![]).concat(&a), Some(a.clone()));
}

#[test]
fn concat_type_mismatch_is_none() {
    let cols = all_types();
    for (i, a) in cols.iter().enumerate() {
        for (j, b) in cols.iter().enumerate() {
            assert_eq!(a.concat(b).is_some(), i == j, "{i} {j}");
        }
    }
    // mismatch is None even when one side is empty
    assert_eq!(Column::Long(vec![]).concat(&Column::Float(vec![])), None);
}

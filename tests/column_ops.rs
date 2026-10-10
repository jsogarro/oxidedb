use oxidedb::types::column::{Column, MAX_ELEMS};
use oxidedb::types::sym::Sym;
use oxidedb::QError;
use proptest::prelude::*;

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
        Column::Long(vec![1, 2]).take(5).unwrap(),
        Column::Long(vec![1, 2, 1, 2, 1])
    );
    assert_eq!(
        Column::Long(vec![1, 2, 3]).take(2).unwrap(),
        Column::Long(vec![1, 2])
    );
    assert_eq!(
        Column::Long(vec![1, 2, 3]).take(3).unwrap(),
        Column::Long(vec![1, 2, 3])
    );
    assert_eq!(
        Column::Long(vec![1, 2, 3]).take(7).unwrap(),
        Column::Long(vec![1, 2, 3, 1, 2, 3, 1])
    );
}

#[test]
fn take_negative_takes_from_the_end_and_wraps() {
    assert_eq!(
        Column::Long(vec![1, 2, 3]).take(-2).unwrap(),
        Column::Long(vec![2, 3])
    );
    assert_eq!(
        Column::Long(vec![1, 2, 3]).take(-3).unwrap(),
        Column::Long(vec![1, 2, 3])
    );
    assert_eq!(
        Column::Long(vec![1, 2]).take(-5).unwrap(),
        Column::Long(vec![2, 1, 2, 1, 2])
    );
    assert_eq!(
        Column::Long(vec![1, 2, 3]).take(-4).unwrap(),
        Column::Long(vec![3, 1, 2, 3])
    );
    assert_eq!(
        Column::Long(vec![1, 2, 3]).take(-1).unwrap(),
        Column::Long(vec![3])
    );
}

#[test]
fn take_zero_is_empty_same_type() {
    for c in all_types() {
        let e = c.take(0).unwrap();
        assert!(e.is_empty());
        assert_eq!(e.type_code(), c.type_code());
    }
}

#[test]
fn take_every_type() {
    assert_eq!(
        Column::Bool(vec![true, false]).take(3).unwrap(),
        Column::Bool(vec![true, false, true])
    );
    assert_eq!(
        Column::Float(vec![1.5, 2.5]).take(-3).unwrap(),
        Column::Float(vec![2.5, 1.5, 2.5])
    );
    assert_eq!(
        Column::Char(vec!['a', 'b']).take(3).unwrap(),
        Column::Char(vec!['a', 'b', 'a'])
    );
    assert_eq!(
        Column::Sym(vec![s("a"), s("b")]).take(-1).unwrap(),
        Column::Sym(vec![s("b")])
    );
}

// ponytail: verify against q — taking from an empty list is assumed to give typed nulls.
#[test]
fn take_from_empty_is_typed_nulls() {
    for c in all_types() {
        let e = empty_like(&c);
        for n in [3, -3] {
            let t = e.take(n).unwrap();
            assert_eq!(t.type_code(), c.type_code());
            assert_eq!(t.len(), 3);
            assert_eq!(t, c.index(&[-1, -1, -1]));
        }
        assert!(e.take(0).unwrap().is_empty());
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

#[test]
fn take_extremes_are_domain_not_panic() {
    let cap = MAX_ELEMS as i64;
    for c in [Column::Long(vec![1, 2]), Column::Long(vec![])] {
        for n in [i64::MIN, i64::MAX, cap + 1, -(cap + 1)] {
            assert_eq!(c.take(n), Err(QError::Domain), "{n}");
        }
    }
}

#[test]
fn take_at_cap_boundary_is_accepted() {
    // A one-byte-per-element column keeps the allocation at 10 MB.
    let c = Column::Bool(vec![true]);
    let cap = MAX_ELEMS as i64;
    assert_eq!(c.take(cap).unwrap().len(), MAX_ELEMS);
    assert_eq!(c.take(-cap).unwrap().len(), MAX_ELEMS);
    assert_eq!(c.take(cap - 1).unwrap().len(), MAX_ELEMS - 1);
}

fn model_take(v: &[i64], n: i64) -> Vec<i64> {
    let len = v.len() as i64;
    let count = n.abs();
    if len == 0 {
        return vec![i64::MIN; count as usize];
    }
    let start = if n < 0 { (len - count % len) % len } else { 0 };
    (0..count)
        .map(|k| v[((start + k) % len) as usize])
        .collect()
}

#[test]
fn take_matches_reference_model() {
    for len in 0..7i64 {
        let v: Vec<i64> = (0..len).map(|i| i * 10 + 1).collect();
        for n in -40..40 {
            assert_eq!(
                Column::Long(v.clone()).take(n).unwrap(),
                Column::Long(model_take(&v, n)),
                "len {len} n {n}"
            );
        }
    }
}

proptest! {
    #[test]
    fn take_len_is_abs_n(v in proptest::collection::vec(any::<i64>(), 0..8), n in -200i64..200) {
        prop_assert_eq!(Column::Long(v).take(n).unwrap().len(), n.unsigned_abs() as usize);
    }

    #[test]
    fn index_out_of_range_is_null(v in proptest::collection::vec(any::<i64>(), 0..8), i in any::<i64>()) {
        let c = Column::Long(v.clone());
        let r = c.index(&[i]);
        let want = usize::try_from(i).ok().and_then(|i| v.get(i).copied()).unwrap_or(i64::MIN);
        prop_assert_eq!(r, Column::Long(vec![want]));
    }

    #[test]
    fn concat_length_is_additive(a in proptest::collection::vec(any::<i64>(), 0..8), b in proptest::collection::vec(any::<i64>(), 0..8)) {
        let r = Column::Long(a.clone()).concat(&Column::Long(b.clone())).unwrap();
        prop_assert_eq!(r.len(), a.len() + b.len());
        prop_assert_eq!(r.index(&(0..a.len() as i64).collect::<Vec<_>>()), Column::Long(a));
    }

    #[test]
    fn concat_mismatch_is_none(a in proptest::collection::vec(any::<i64>(), 0..4), b in proptest::collection::vec(any::<bool>(), 0..4)) {
        prop_assert!(Column::Long(a).concat(&Column::Bool(b)).is_none());
    }
}

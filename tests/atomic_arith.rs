use oxidedb::language::ast::Verb;
use oxidedb::language::ops::{dyad, monad_neg};
use oxidedb::types::column::Column;
use oxidedb::types::sym::Sym;
use oxidedb::{Atom, Interpreter, QError, Value};
use proptest::prelude::*;
use std::rc::Rc;

const NULL: i64 = i64::MIN;
const ARITH: [Verb; 4] = [Verb::Add, Verb::Subtract, Verb::Multiply, Verb::Divide];

fn l(n: i64) -> Value {
    Value::Atom(Atom::Integer(n))
}
fn f(x: f64) -> Value {
    Value::Atom(Atom::Float(x))
}
fn b(x: bool) -> Value {
    Value::Atom(Atom::Boolean(x))
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
fn cv(s: &str) -> Value {
    Value::Vector(Rc::new(Column::Char(s.chars().collect())))
}
fn list(items: Vec<Value>) -> Value {
    Value::List(Rc::new(items))
}
fn add(x: &Value, y: &Value) -> Result<Value, QError> {
    dyad(Verb::Add, x, y)
}
fn sub(x: &Value, y: &Value) -> Result<Value, QError> {
    dyad(Verb::Subtract, x, y)
}
fn mul(x: &Value, y: &Value) -> Result<Value, QError> {
    dyad(Verb::Multiply, x, y)
}
fn div(x: &Value, y: &Value) -> Result<Value, QError> {
    dyad(Verb::Divide, x, y)
}

// Atoms behave exactly as before the kernel existed.
#[test]
fn arith_atoms_unchanged() {
    assert_eq!(add(&l(2), &l(3)), Ok(l(5)));
    assert_eq!(sub(&l(2), &l(3)), Ok(l(-1)));
    assert_eq!(mul(&l(2), &l(3)), Ok(l(6)));
    assert_eq!(div(&l(6), &l(3)), Ok(f(2.0)));
    assert_eq!(add(&f(1.5), &f(2.0)), Ok(f(3.5)));
    assert_eq!(sub(&f(1.5), &f(2.0)), Ok(f(-0.5)));
    assert_eq!(mul(&f(1.5), &f(2.0)), Ok(f(3.0)));
    assert_eq!(div(&f(1.0), &f(4.0)), Ok(f(0.25)));
    // IEEE: x%0 is not an error
    assert_eq!(div(&l(1), &l(0)), Ok(f(f64::INFINITY)));
    assert_eq!(div(&l(-1), &l(0)), Ok(f(f64::NEG_INFINITY)));
    assert_eq!(div(&l(0), &l(0)), Ok(f(f64::NAN)));
    assert_eq!(div(&f(1.0), &f(0.0)), Ok(f(f64::INFINITY)));
}

#[test]
fn arith_atom_vector_broadcast() {
    assert_eq!(add(&l(1), &lv(&[1, 2, 3])), Ok(lv(&[2, 3, 4])));
    assert_eq!(mul(&lv(&[1, 2, 3]), &l(2)), Ok(lv(&[2, 4, 6])));
    // operand order matters for - and %
    assert_eq!(sub(&l(10), &lv(&[1, 2, 3])), Ok(lv(&[9, 8, 7])));
    assert_eq!(sub(&lv(&[1, 2, 3]), &l(10)), Ok(lv(&[-9, -8, -7])));
    assert_eq!(div(&l(12), &lv(&[2, 4])), Ok(fv(&[6.0, 3.0])));
    assert_eq!(div(&lv(&[2, 4]), &l(8)), Ok(fv(&[0.25, 0.5])));
    assert_eq!(add(&fv(&[1.0, 2.0]), &f(0.5)), Ok(fv(&[1.5, 2.5])));
    assert_eq!(sub(&f(0.5), &fv(&[1.0, 2.0])), Ok(fv(&[-0.5, -1.5])));
    assert_eq!(mul(&f(2.0), &fv(&[1.0, 2.5])), Ok(fv(&[2.0, 5.0])));
}

#[test]
fn arith_vector_pairwise() {
    let x = lv(&[10, 20, 30]);
    let y = lv(&[1, 2, 3]);
    assert_eq!(add(&x, &y), Ok(lv(&[11, 22, 33])));
    assert_eq!(sub(&x, &y), Ok(lv(&[9, 18, 27])));
    assert_eq!(mul(&x, &y), Ok(lv(&[10, 40, 90])));
    assert_eq!(div(&x, &y), Ok(fv(&[10.0, 10.0, 10.0])));
    assert_eq!(sub(&y, &x), Ok(lv(&[-9, -18, -27])));
    let p = fv(&[1.0, 2.0]);
    let q = fv(&[4.0, 8.0]);
    assert_eq!(add(&p, &q), Ok(fv(&[5.0, 10.0])));
    assert_eq!(sub(&p, &q), Ok(fv(&[-3.0, -6.0])));
    assert_eq!(mul(&p, &q), Ok(fv(&[4.0, 16.0])));
    assert_eq!(div(&p, &q), Ok(fv(&[0.25, 0.25])));
}

#[test]
fn arith_length_mismatch() {
    for verb in ARITH {
        let e = Err(QError::Length);
        assert_eq!(dyad(verb, &lv(&[1, 2]), &lv(&[1, 2, 3])), e);
        assert_eq!(dyad(verb, &lv(&[1, 2, 3]), &lv(&[1, 2])), e);
        assert_eq!(dyad(verb, &fv(&[1.0]), &fv(&[1.0, 2.0])), e);
        assert_eq!(dyad(verb, &lv(&[1]), &fv(&[1.0, 2.0])), e);
        assert_eq!(dyad(verb, &fv(&[1.0, 2.0]), &lv(&[1])), e);
        assert_eq!(dyad(verb, &lv(&[]), &lv(&[1])), e);
        assert_eq!(dyad(verb, &lv(&[1]), &lv(&[])), e);
        assert_eq!(dyad(verb, &bv(&[true]), &bv(&[true, false])), e);
        // lists: item count must match too
        let two = list(vec![l(1), lv(&[2, 3])]);
        assert_eq!(dyad(verb, &two, &lv(&[1, 2, 3])), e);
        assert_eq!(dyad(verb, &lv(&[1, 2, 3]), &two), e);
        assert_eq!(dyad(verb, &two, &list(vec![l(1)])), e);
        assert_eq!(dyad(verb, &list(vec![]), &lv(&[1])), e);
    }
}

#[test]
fn arith_bool_promotes_to_long() {
    assert_eq!(add(&b(true), &b(true)), Ok(l(2)));
    assert_eq!(add(&b(true), &l(1)), Ok(l(2)));
    assert_eq!(add(&l(1), &b(true)), Ok(l(2)));
    assert_eq!(mul(&l(2), &b(true)), Ok(l(2)));
    assert_eq!(mul(&l(2), &b(false)), Ok(l(0)));
    assert_eq!(sub(&b(false), &b(true)), Ok(l(-1)));
    assert_eq!(sub(&b(true), &b(false)), Ok(l(1)));
    assert_eq!(div(&b(true), &b(true)), Ok(f(1.0)));
    assert_eq!(div(&b(true), &b(false)), Ok(f(f64::INFINITY)));
    // bool with float is float
    assert_eq!(add(&b(true), &f(0.5)), Ok(f(1.5)));
    assert_eq!(mul(&f(0.5), &b(true)), Ok(f(0.5)));
    // vectors
    assert_eq!(
        add(&bv(&[true, false]), &bv(&[true, true])),
        Ok(lv(&[2, 1]))
    );
    assert_eq!(add(&bv(&[true, false]), &l(1)), Ok(lv(&[2, 1])));
    assert_eq!(add(&l(1), &bv(&[true, false])), Ok(lv(&[2, 1])));
    assert_eq!(mul(&bv(&[true, false]), &b(true)), Ok(lv(&[1, 0])));
    assert_eq!(sub(&bv(&[true, false]), &lv(&[5, 5])), Ok(lv(&[-4, -5])));
    assert_eq!(add(&bv(&[true, false]), &f(0.5)), Ok(fv(&[1.5, 0.5])));
    assert_eq!(
        div(&bv(&[true, false]), &bv(&[true, true])),
        Ok(fv(&[1.0, 0.0]))
    );
    assert_eq!(add(&bv(&[]), &l(1)), Ok(lv(&[])));
}

#[test]
fn arith_long_float_promotes() {
    assert_eq!(add(&l(1), &f(0.5)), Ok(f(1.5)));
    assert_eq!(add(&f(0.5), &l(1)), Ok(f(1.5)));
    assert_eq!(sub(&l(1), &f(0.5)), Ok(f(0.5)));
    assert_eq!(sub(&f(0.5), &l(1)), Ok(f(-0.5)));
    assert_eq!(mul(&l(3), &f(0.5)), Ok(f(1.5)));
    assert_eq!(div(&l(1), &f(4.0)), Ok(f(0.25)));
    assert_eq!(div(&f(1.0), &l(4)), Ok(f(0.25)));
    assert_eq!(add(&lv(&[1, 2]), &f(0.5)), Ok(fv(&[1.5, 2.5])));
    assert_eq!(add(&f(0.5), &lv(&[1, 2])), Ok(fv(&[1.5, 2.5])));
    assert_eq!(add(&lv(&[1, 2]), &fv(&[0.5, 0.25])), Ok(fv(&[1.5, 2.25])));
    assert_eq!(add(&fv(&[0.5, 0.25]), &lv(&[1, 2])), Ok(fv(&[1.5, 2.25])));
    assert_eq!(sub(&lv(&[1, 2]), &fv(&[0.5, 0.25])), Ok(fv(&[0.5, 1.75])));
    assert_eq!(sub(&fv(&[0.5, 0.25]), &lv(&[1, 2])), Ok(fv(&[-0.5, -1.75])));
    assert_eq!(mul(&lv(&[2, 4]), &fv(&[0.5, 0.25])), Ok(fv(&[1.0, 1.0])));
    assert_eq!(div(&lv(&[1, 2]), &fv(&[4.0, 4.0])), Ok(fv(&[0.25, 0.5])));
    assert_eq!(div(&fv(&[1.0, 2.0]), &lv(&[4, 4])), Ok(fv(&[0.25, 0.5])));
    // mixed long/float with a long that does not fit a float exactly still converts
    assert_eq!(add(&l(i64::MAX), &f(0.0)), Ok(f(i64::MAX as f64)));
}

#[test]
fn arith_divide_vectors_float() {
    let r = div(&lv(&[1, 2]), &lv(&[2, 4])).unwrap();
    assert_eq!(r.type_code(), 9);
    assert_eq!(r, fv(&[0.5, 0.5]));
    assert_eq!(
        div(&lv(&[1, 0, -1]), &l(0)),
        Ok(fv(&[f64::INFINITY, f64::NAN, f64::NEG_INFINITY]))
    );
    assert_eq!(div(&lv(&[]), &l(1)), Ok(fv(&[])));
    assert_eq!(div(&l(1), &lv(&[])), Ok(fv(&[])));
    assert_eq!(div(&lv(&[]), &lv(&[])), Ok(fv(&[])));
    assert_eq!(div(&fv(&[]), &fv(&[])), Ok(fv(&[])));
}

#[test]
fn arith_null_propagates() {
    // atoms
    for verb in [Verb::Add, Verb::Subtract, Verb::Multiply] {
        assert_eq!(dyad(verb, &l(NULL), &l(1)), Ok(l(NULL)), "{verb:?}");
        assert_eq!(dyad(verb, &l(1), &l(NULL)), Ok(l(NULL)), "{verb:?}");
        assert_eq!(dyad(verb, &l(NULL), &l(NULL)), Ok(l(NULL)), "{verb:?}");
        // per element, either side
        let r = dyad(verb, &lv(&[1, NULL, 3]), &l(2)).unwrap();
        let want = [
            dyad(verb, &l(1), &l(2)).unwrap(),
            l(NULL),
            dyad(verb, &l(3), &l(2)).unwrap(),
        ];
        assert_eq!(r, Value::from_items(want.to_vec()));
        assert_eq!(dyad(verb, &l(2), &lv(&[1, NULL])).unwrap().type_code(), 7);
        let r = dyad(verb, &l(2), &lv(&[1, NULL])).unwrap();
        assert_eq!(
            r,
            Value::from_items(vec![dyad(verb, &l(2), &l(1)).unwrap(), l(NULL)])
        );
        // null in the other vector operand
        let r = dyad(verb, &lv(&[1, 2]), &lv(&[NULL, 1])).unwrap();
        assert_eq!(
            r,
            Value::from_items(vec![l(NULL), dyad(verb, &l(2), &l(1)).unwrap()])
        );
        let r = dyad(verb, &lv(&[NULL, 2]), &lv(&[1, 1])).unwrap();
        assert_eq!(
            r,
            Value::from_items(vec![l(NULL), dyad(verb, &l(2), &l(1)).unwrap()])
        );
    }
    // with a float the null becomes 0n
    assert_eq!(add(&l(NULL), &f(1.0)), Ok(f(f64::NAN)));
    assert_eq!(add(&f(1.0), &l(NULL)), Ok(f(f64::NAN)));
    assert_eq!(mul(&f(2.0), &l(NULL)), Ok(f(f64::NAN)));
    assert_eq!(sub(&l(NULL), &f(1.0)), Ok(f(f64::NAN)));
    assert_eq!(add(&lv(&[NULL, 1]), &f(1.0)), Ok(fv(&[f64::NAN, 2.0])));
    assert_eq!(
        add(&fv(&[1.0, 2.0]), &lv(&[NULL, 1])),
        Ok(fv(&[f64::NAN, 3.0]))
    );
    // 0n * 2 stays 0n
    assert_eq!(mul(&f(f64::NAN), &l(2)), Ok(f(f64::NAN)));
    assert_eq!(mul(&fv(&[f64::NAN, 1.0]), &l(2)), Ok(fv(&[f64::NAN, 2.0])));
    // % yields 0n for a long null on either side
    assert_eq!(div(&l(NULL), &l(2)), Ok(f(f64::NAN)));
    assert_eq!(div(&l(2), &l(NULL)), Ok(f(f64::NAN)));
    assert_eq!(div(&lv(&[NULL, 4]), &l(2)), Ok(fv(&[f64::NAN, 2.0])));
    assert_eq!(div(&l(8), &lv(&[NULL, 4])), Ok(fv(&[f64::NAN, 2.0])));
    assert_eq!(div(&lv(&[8, 8]), &lv(&[NULL, 4])), Ok(fv(&[f64::NAN, 2.0])));
    // a long null is not a number: 0N-1 is not i64::MIN-1 overflow
    assert_eq!(sub(&l(NULL), &l(1)), Ok(l(NULL)));
    assert_eq!(add(&l(NULL), &l(-1)), Ok(l(NULL)));
}

#[test]
fn arith_symbol_is_type_error() {
    let sym = Value::Atom(Atom::Symbol(Sym::intern("a")));
    let chr = Value::Atom(Atom::Character('a'));
    let date = Value::Atom(Atom::NullDate);
    for verb in ARITH {
        let e = Err(QError::Type);
        for bad in [&sym, &chr, &date, &sv(&["a"]), &cv("a")] {
            assert_eq!(dyad(verb, bad, &l(1)), e, "{verb:?} {bad:?}");
            assert_eq!(dyad(verb, &l(1), bad), e, "{verb:?} {bad:?}");
            assert_eq!(dyad(verb, &f(1.0), bad), e, "{verb:?} {bad:?}");
            assert_eq!(dyad(verb, &lv(&[1]), bad), e, "{verb:?} {bad:?}");
            assert_eq!(dyad(verb, bad, &fv(&[1.0])), e, "{verb:?} {bad:?}");
            assert_eq!(dyad(verb, bad, &b(true)), e, "{verb:?} {bad:?}");
            assert_eq!(dyad(verb, &bv(&[true]), bad), e, "{verb:?} {bad:?}");
        }
        // empty vectors: a type error against an atom, a length error against
        // a one-item vector (lengths are checked first, as in q)
        for empty in [&sv(&[]), &cv("")] {
            assert_eq!(dyad(verb, empty, &l(1)), e, "{verb:?}");
            assert_eq!(dyad(verb, &l(1), empty), e, "{verb:?}");
            assert_eq!(dyad(verb, empty, &lv(&[])), e, "{verb:?}");
            assert_eq!(
                dyad(verb, &lv(&[1]), empty),
                Err(QError::Length),
                "{verb:?}"
            );
        }
        assert_eq!(dyad(verb, &sym, &sym), e);
        assert_eq!(dyad(verb, &chr, &chr), e);
        assert_eq!(dyad(verb, &sv(&["a"]), &sv(&["b"])), e);
        assert_eq!(dyad(verb, &cv("ab"), &cv("cd")), e);
        // nested inside a general list
        assert_eq!(dyad(verb, &list(vec![l(1), sv(&["a"])]), &l(1)), e);
        assert_eq!(dyad(verb, &list(vec![l(1), sym.clone()]), &l(1)), e);
    }
}

#[test]
fn arith_general_list_recurses() {
    let nested = list(vec![l(1), lv(&[2, 3])]);
    assert_eq!(add(&nested, &l(10)), Ok(list(vec![l(11), lv(&[12, 13])])));
    assert_eq!(add(&l(10), &nested), Ok(list(vec![l(11), lv(&[12, 13])])));
    assert_eq!(sub(&l(10), &nested), Ok(list(vec![l(9), lv(&[8, 7])])));
    assert_eq!(sub(&nested, &l(10)), Ok(list(vec![l(-9), lv(&[-8, -7])])));
    assert_eq!(mul(&nested, &l(2)), Ok(list(vec![l(2), lv(&[4, 6])])));
    assert_eq!(div(&nested, &l(2)), Ok(list(vec![f(0.5), fv(&[1.0, 1.5])])));
    // list with list, pairwise
    let other = list(vec![l(10), lv(&[20, 30])]);
    assert_eq!(add(&nested, &other), Ok(list(vec![l(11), lv(&[22, 33])])));
    assert_eq!(sub(&other, &nested), Ok(list(vec![l(9), lv(&[18, 27])])));
    // list with vector, both orders, item i against element i
    assert_eq!(
        add(&nested, &lv(&[100, 200])),
        Ok(list(vec![l(101), lv(&[202, 203])]))
    );
    assert_eq!(
        sub(&lv(&[100, 200]), &nested),
        Ok(list(vec![l(99), lv(&[198, 197])]))
    );
    // deeper nesting
    let deep = list(vec![list(vec![l(1), lv(&[2, 3])]), l(4)]);
    assert_eq!(
        add(&deep, &l(1)),
        Ok(list(vec![list(vec![l(2), lv(&[3, 4])]), l(5)]))
    );
    // the result is normalised: a hand-built list of same-typed atoms is a vector
    assert_eq!(add(&list(vec![l(1), l(2)]), &l(1)), Ok(lv(&[2, 3])));
    assert_eq!(
        add(&list(vec![l(1), f(2.0)]), &l(1)),
        Ok(list(vec![l(2), f(3.0)]))
    );
    // empty general list stays a general list
    assert_eq!(add(&list(vec![]), &l(1)), Ok(list(vec![])));
    assert_eq!(add(&l(1), &list(vec![])), Ok(list(vec![])));
    assert_eq!(add(&list(vec![]), &list(vec![])), Ok(list(vec![])));
    assert_eq!(add(&list(vec![]), &lv(&[])), Ok(list(vec![])));
    // errors inside items surface
    assert_eq!(
        add(&list(vec![l(i64::MAX), l(1)]), &l(1)),
        Err(QError::Overflow)
    );
    assert_eq!(add(&nested, &lv(&[1, 2, 3])), Err(QError::Length));
}

// Results are renormalised: same-typed items collapse into a typed vector,
// whichever list arm produced them.
#[test]
fn arith_list_results_collapse_to_vectors() {
    // list with list
    assert_eq!(
        add(&list(vec![l(1), f(2.5)]), &list(vec![f(0.5), l(1)])),
        Ok(fv(&[1.5, 3.5]))
    );
    assert_eq!(
        add(&list(vec![l(1), l(2)]), &list(vec![l(1), l(2)])),
        Ok(lv(&[2, 4]))
    );
    // list with vector, both orders
    assert_eq!(
        add(&list(vec![l(1), f(2.5)]), &fv(&[0.5, 1.0])),
        Ok(fv(&[1.5, 3.5]))
    );
    assert_eq!(
        add(&fv(&[0.5, 1.0]), &list(vec![l(1), f(2.5)])),
        Ok(fv(&[1.5, 3.5]))
    );
    assert_eq!(add(&list(vec![l(1), l(2)]), &lv(&[1, 2])), Ok(lv(&[2, 4])));
    assert_eq!(add(&lv(&[1, 2]), &list(vec![l(1), l(2)])), Ok(lv(&[2, 4])));
    // list with atom, both orders
    assert_eq!(add(&list(vec![l(1), l(2)]), &l(1)), Ok(lv(&[2, 3])));
    assert_eq!(add(&l(1), &list(vec![l(1), l(2)])), Ok(lv(&[2, 3])));
    assert_eq!(
        add(&l(1), &list(vec![l(1), f(2.0)])),
        Ok(list(vec![l(2), f(3.0)]))
    );
    // mixed long and float items stay general (type 0), as in q ...
    let mixed = list(vec![l(1), f(2.5)]);
    assert_eq!(add(&mixed, &l(1)), Ok(list(vec![l(2), f(3.5)])));
    assert_eq!(add(&mixed, &l(1)).unwrap().type_code(), 0);
    // ... until the atom is a float
    assert_eq!(add(&mixed, &f(1.0)), Ok(fv(&[2.0, 3.5])));
    assert_eq!(add(&f(1.0), &mixed), Ok(fv(&[2.0, 3.5])));
}

#[test]
fn arith_overflow_not_at_the_boundary() {
    let max = i64::MAX;
    // far past the limit, where a wrapping operation would land on a valid value
    assert_eq!(add(&lv(&[max, max]), &l(max)), Err(QError::Overflow));
    assert_eq!(add(&l(max), &lv(&[max, max])), Err(QError::Overflow));
    assert_eq!(
        add(&lv(&[max, max]), &lv(&[max, max])),
        Err(QError::Overflow)
    );
    assert_eq!(add(&l(max), &l(max)), Err(QError::Overflow));
    assert_eq!(add(&l(-max), &l(-max)), Err(QError::Overflow));
    assert_eq!(sub(&lv(&[-max, -max]), &l(max)), Err(QError::Overflow));
    assert_eq!(sub(&l(max), &lv(&[-max, -max])), Err(QError::Overflow));
    assert_eq!(sub(&lv(&[max]), &lv(&[-max])), Err(QError::Overflow));
    assert_eq!(mul(&lv(&[max, max]), &l(max)), Err(QError::Overflow));
    assert_eq!(mul(&l(-max), &lv(&[max, max])), Err(QError::Overflow));
    assert_eq!(mul(&lv(&[1 << 40]), &lv(&[1 << 40])), Err(QError::Overflow));
    assert_eq!(mul(&l(1 << 40), &l(1 << 40)), Err(QError::Overflow));
}

#[test]
fn arith_overflow_in_vector() {
    let max = i64::MAX;
    assert_eq!(add(&lv(&[1, max]), &l(1)), Err(QError::Overflow));
    assert_eq!(add(&l(1), &lv(&[1, max])), Err(QError::Overflow));
    assert_eq!(add(&lv(&[1, max]), &lv(&[1, 1])), Err(QError::Overflow));
    assert_eq!(sub(&lv(&[1, -max]), &l(2)), Err(QError::Overflow));
    assert_eq!(sub(&l(-2), &lv(&[1, max])), Err(QError::Overflow));
    assert_eq!(sub(&lv(&[1, -max]), &lv(&[1, 2])), Err(QError::Overflow));
    assert_eq!(mul(&lv(&[1, max]), &l(2)), Err(QError::Overflow));
    assert_eq!(mul(&l(2), &lv(&[1, max])), Err(QError::Overflow));
    assert_eq!(mul(&lv(&[1, max]), &lv(&[1, 2])), Err(QError::Overflow));
    // a non-null computation that lands exactly on i64::MIN is overflow, as for atoms
    assert_eq!(add(&l(-max), &l(-1)), Err(QError::Overflow));
    assert_eq!(sub(&l(-max), &l(1)), Err(QError::Overflow));
    assert_eq!(mul(&l(1 << 32), &l(-(1 << 31))), Err(QError::Overflow));
    assert_eq!(add(&lv(&[-max]), &l(-1)), Err(QError::Overflow));
    assert_eq!(sub(&lv(&[-max]), &l(1)), Err(QError::Overflow));
    assert_eq!(mul(&lv(&[1 << 32]), &l(-(1 << 31))), Err(QError::Overflow));
    // one below the limit is fine
    assert_eq!(add(&lv(&[max - 1]), &l(1)), Ok(lv(&[max])));
    assert_eq!(sub(&lv(&[-max + 1]), &l(1)), Ok(lv(&[-max])));
    // overflow wins over a later null, nulls never overflow
    assert_eq!(add(&lv(&[max, NULL]), &l(1)), Err(QError::Overflow));
    assert_eq!(add(&lv(&[NULL, NULL]), &l(1)), Ok(lv(&[NULL, NULL])));
    // overflow inside bool-promoted arithmetic
    assert_eq!(add(&lv(&[max]), &bv(&[true])), Err(QError::Overflow));
    // % never overflows
    assert_eq!(div(&lv(&[max]), &l(-1)), Ok(fv(&[max as f64 / -1.0])));
}

#[test]
fn arith_empty_vectors() {
    for verb in [Verb::Add, Verb::Subtract, Verb::Multiply] {
        assert_eq!(dyad(verb, &lv(&[]), &l(1)), Ok(lv(&[])));
        assert_eq!(dyad(verb, &l(1), &lv(&[])), Ok(lv(&[])));
        assert_eq!(dyad(verb, &lv(&[]), &lv(&[])), Ok(lv(&[])));
        assert_eq!(dyad(verb, &fv(&[]), &l(1)), Ok(fv(&[])));
        assert_eq!(dyad(verb, &lv(&[]), &f(1.0)), Ok(fv(&[])));
        assert_eq!(dyad(verb, &lv(&[]), &fv(&[])), Ok(fv(&[])));
        assert_eq!(dyad(verb, &fv(&[]), &lv(&[])), Ok(fv(&[])));
        assert_eq!(dyad(verb, &bv(&[]), &bv(&[])), Ok(lv(&[])));
    }
    assert_eq!(add(&lv(&[]), &fv(&[1.0])), Err(QError::Length));
}

#[test]
fn arith_negate() {
    assert_eq!(monad_neg(&l(5)), Ok(l(-5)));
    assert_eq!(monad_neg(&f(1.5)), Ok(f(-1.5)));
    assert_eq!(monad_neg(&l(NULL)), Ok(l(NULL)));
    assert_eq!(monad_neg(&f(f64::NAN)), Ok(f(f64::NAN)));
    assert_eq!(monad_neg(&lv(&[1, -2, NULL])), Ok(lv(&[-1, 2, NULL])));
    assert_eq!(
        monad_neg(&fv(&[1.5, -2.0, f64::NAN])),
        Ok(fv(&[-1.5, 2.0, f64::NAN]))
    );
    assert_eq!(monad_neg(&lv(&[])), Ok(lv(&[])));
    assert_eq!(monad_neg(&fv(&[])), Ok(fv(&[])));
    assert_eq!(
        monad_neg(&list(vec![l(1), lv(&[2, 3])])),
        Ok(list(vec![l(-1), lv(&[-2, -3])]))
    );
    assert_eq!(monad_neg(&list(vec![])), Ok(list(vec![])));
    assert_eq!(monad_neg(&list(vec![l(1), l(2)])), Ok(lv(&[-1, -2])));
    // -(MIN+1) is fine; there is no non-null value whose negation is MIN
    assert_eq!(monad_neg(&lv(&[NULL + 1])), Ok(lv(&[i64::MAX])));
    assert_eq!(monad_neg(&lv(&[i64::MAX])), Ok(lv(&[NULL + 1])));
    // a boolean counts as a long, as in the dyadic verbs
    assert_eq!(monad_neg(&b(true)), Ok(l(-1)));
    assert_eq!(monad_neg(&b(false)), Ok(l(0)));
    assert_eq!(monad_neg(&bv(&[true, false])), Ok(lv(&[-1, 0])));
    assert_eq!(monad_neg(&bv(&[])), Ok(lv(&[])));
    assert_eq!(
        monad_neg(&list(vec![b(true), lv(&[2])])),
        Ok(list(vec![l(-1), lv(&[-2])]))
    );
    // symbols, chars and temporals are not negatable
    assert_eq!(monad_neg(&sv(&["a"])), Err(QError::Type));
    assert_eq!(monad_neg(&cv("a")), Err(QError::Type));
    assert_eq!(monad_neg(&sv(&[])), Err(QError::Type));
    assert_eq!(monad_neg(&Value::Atom(Atom::NullDate)), Err(QError::Type));
    assert_eq!(monad_neg(&list(vec![l(1), sv(&["a"])])), Err(QError::Type));
}

#[test]
fn arith_other_verbs_stay_nyi() {
    let verb = Verb::Key;
    let e = Err(QError::Nyi(verb.symbol().into()));
    assert_eq!(dyad(verb, &l(1), &l(2)), e);
    assert_eq!(dyad(verb, &lv(&[1]), &l(2)), e);
    assert_eq!(dyad(verb, &l(1), &list(vec![])), e);
}

// The language cannot write vector literals yet, so bind them from the host.
#[test]
fn arith_through_the_interpreter() {
    let mut i = Interpreter::new();
    i.set("v", lv(&[1, 2, 3]));
    i.set("w", lv(&[10, 20, 30]));
    i.set("x", fv(&[0.5, 1.5]));
    i.set("n", list(vec![l(1), lv(&[2, 3])]));
    let mut ev = |s: &str| i.eval_line(s).map(|o| o.unwrap());
    assert_eq!(ev("v+1"), Ok(lv(&[2, 3, 4])));
    assert_eq!(ev("1+v"), Ok(lv(&[2, 3, 4])));
    assert_eq!(ev("10-v"), Ok(lv(&[9, 8, 7])));
    assert_eq!(ev("v-10"), Ok(lv(&[-9, -8, -7])));
    assert_eq!(ev("v*v"), Ok(lv(&[1, 4, 9])));
    assert_eq!(ev("w-v"), Ok(lv(&[9, 18, 27])));
    assert_eq!(ev("-v"), Ok(lv(&[-1, -2, -3])));
    assert_eq!(ev("- v"), Ok(lv(&[-1, -2, -3])));
    assert_eq!(ev("v%2"), Ok(fv(&[0.5, 1.0, 1.5])));
    assert_eq!(ev("v+x"), Err(QError::Length));
    assert_eq!(ev("v+0.5"), Ok(fv(&[1.5, 2.5, 3.5])));
    assert_eq!(ev("n+10"), Ok(list(vec![l(11), lv(&[12, 13])])));
    assert_eq!(ev("-n"), Ok(list(vec![l(-1), lv(&[-2, -3])])));
    assert_eq!(ev("v+1b"), Ok(lv(&[2, 3, 4])));
    assert_eq!(ev("1b+1"), Ok(l(2)));
    assert_eq!(ev("1b+1b"), Ok(l(2)));
    assert_eq!(ev("-(1b)"), Ok(l(-1)));
    assert_eq!(ev("- 1b"), Ok(l(-1)));
    assert_eq!(ev("-1b"), Ok(l(-1)));
    assert_eq!(ev("0-1b"), Ok(l(-1)));
    assert_eq!(ev("2*1b"), Ok(l(2)));
    assert_eq!(ev("1+\"a\""), Err(QError::Type));
    // right to left: the right operand is evaluated first
    assert_eq!(ev("v+a:1"), Ok(lv(&[2, 3, 4])));
    assert_eq!(ev("a"), Ok(l(1)));
    // assignment keeps the vector result
    assert_eq!(ev("u:v*2"), Ok(lv(&[2, 4, 6])));
    assert_eq!(ev("u+1"), Ok(lv(&[3, 5, 7])));
    assert_eq!(ev("v+9223372036854775807"), Err(QError::Overflow));
}

fn small() -> impl Strategy<Value = i64> {
    prop_oneof![
        -1000i64..1000,
        Just(i64::MAX),
        Just(NULL),
        Just(NULL + 1),
        any::<i64>()
    ]
}
fn arith_verb() -> impl Strategy<Value = Verb> {
    prop_oneof![
        Just(Verb::Add),
        Just(Verb::Subtract),
        Just(Verb::Multiply),
        Just(Verb::Divide)
    ]
}

proptest! {
    // vector∘atom is the atom operation mapped over the elements.
    #[test]
    fn arith_prop_vector_atom_is_mapped_atom_op(
        verb in arith_verb(), xs in prop::collection::vec(small(), 1..8), a in small()
    ) {
        let want: Result<Vec<Value>, QError> =
            xs.iter().map(|&x| dyad(verb, &l(x), &l(a))).collect();
        let want = want.map(Value::from_items);
        prop_assert_eq!(dyad(verb, &lv(&xs), &l(a)), want.clone());
        let want_rev: Result<Vec<Value>, QError> =
            xs.iter().map(|&x| dyad(verb, &l(a), &l(x))).collect();
        prop_assert_eq!(dyad(verb, &l(a), &lv(&xs)), want_rev.map(Value::from_items));
    }

    #[test]
    fn arith_prop_float_vector_atom_is_mapped_atom_op(
        verb in arith_verb(),
        xs in prop::collection::vec(prop_oneof![-1e6f64..1e6, Just(f64::NAN)], 1..8),
        a in prop_oneof![-1e6f64..1e6, Just(f64::NAN)],
        k in small(),
    ) {
        let want: Vec<Value> = xs.iter().map(|&x| dyad(verb, &f(x), &f(a)).unwrap()).collect();
        prop_assert_eq!(dyad(verb, &fv(&xs), &f(a)), Ok(Value::from_items(want)));
        // mixed with a long atom
        let want: Vec<Value> = xs.iter().map(|&x| dyad(verb, &f(x), &l(k)).unwrap()).collect();
        prop_assert_eq!(dyad(verb, &fv(&xs), &l(k)), Ok(Value::from_items(want)));
        let want: Vec<Value> = xs.iter().map(|&x| dyad(verb, &l(k), &f(x)).unwrap()).collect();
        prop_assert_eq!(dyad(verb, &l(k), &fv(&xs)), Ok(Value::from_items(want)));
    }

    // vector∘vector is the atom operation mapped pairwise.
    #[test]
    fn arith_prop_pairwise_is_mapped_atom_op(
        verb in arith_verb(),
        pairs in prop::collection::vec((small(), small()), 1..8),
    ) {
        let (xs, ys): (Vec<i64>, Vec<i64>) = pairs.into_iter().unzip();
        let want: Result<Vec<Value>, QError> = xs
            .iter()
            .zip(&ys)
            .map(|(&x, &y)| dyad(verb, &l(x), &l(y)))
            .collect();
        prop_assert_eq!(dyad(verb, &lv(&xs), &lv(&ys)), want.map(Value::from_items));
    }

    #[test]
    fn arith_prop_commutative_for_add_and_mul(a in small(), b in small()) {
        for verb in [Verb::Add, Verb::Multiply] {
            prop_assert_eq!(dyad(verb, &l(a), &l(b)), dyad(verb, &l(b), &l(a)));
            prop_assert_eq!(dyad(verb, &lv(&[a, b]), &l(a)), dyad(verb, &l(a), &lv(&[a, b])));
            prop_assert_eq!(dyad(verb, &lv(&[a]), &lv(&[b])), dyad(verb, &lv(&[b]), &lv(&[a])));
        }
    }

    #[test]
    fn arith_prop_broadcast_preserves_length(
        verb in arith_verb(), xs in prop::collection::vec(-1000i64..1000, 0..16), a in -1000i64..1000
    ) {
        for r in [dyad(verb, &lv(&xs), &l(a)), dyad(verb, &l(a), &lv(&xs))] {
            match r.unwrap() {
                Value::Vector(c) => prop_assert_eq!(c.len(), xs.len()),
                other => prop_assert!(false, "expected a vector, got {other:?}"),
            }
        }
    }

    #[test]
    fn arith_prop_mismatched_lengths_are_length_errors(
        verb in arith_verb(), n in 0usize..6, m in 0usize..6
    ) {
        prop_assume!(n != m);
        prop_assert_eq!(dyad(verb, &lv(&vec![1; n]), &lv(&vec![1; m])), Err(QError::Length));
    }
}

// #86: q checks lengths before element types (`1 2 3+`a`b` is 'length).
#[test]
fn arith_length_is_checked_before_type() {
    let sym_atom = Value::Atom(Atom::Symbol(Sym::intern("a")));
    for verb in ARITH {
        assert_eq!(
            dyad(verb, &lv(&[1, 2, 3]), &sv(&["a", "b"])),
            Err(QError::Length)
        );
        assert_eq!(dyad(verb, &lv(&[1, 2, 3]), &cv("")), Err(QError::Length));
        assert_eq!(
            dyad(verb, &sv(&["a", "b"]), &fv(&[1.0, 2.0, 3.0])),
            Err(QError::Length)
        );
        assert_eq!(
            dyad(verb, &cv("ab"), &sv(&["a", "b", "c"])),
            Err(QError::Length)
        );
        // equal lengths still report the type
        assert_eq!(
            dyad(verb, &lv(&[1, 2]), &sv(&["a", "b"])),
            Err(QError::Type)
        );
        // an atom side has no length to mismatch
        assert_eq!(dyad(verb, &lv(&[1, 2, 3]), &sym_atom), Err(QError::Type));
    }
}

// Deliberate deviation: lengths are checked first for every verb. q reports
// 'type for `1 2 3-`a`b` (subtract with a symbol vector on the right) but
// 'length for every other verb and for the mirrored `` `a`b-1 2 3 ``.
#[test]
fn arith_subtract_symbol_vector_length_first() {
    assert_eq!(sub(&lv(&[1, 2, 3]), &sv(&["a", "b"])), Err(QError::Length));
    assert_eq!(sub(&sv(&["a", "b"]), &lv(&[1, 2, 3])), Err(QError::Length));
}

//! `+ - * %` over atoms, vectors and general lists, atomic as in q.
//! Bool promotes to long; long with float is float; `%` is always float.

use super::{atomic, Flat};
use crate::error::{QError, QResult};
use crate::language::ast::Verb;
use crate::types::atom::Atom;
use crate::types::column::Column;
use crate::types::value::Value;
use std::borrow::Cow;
use std::rc::Rc;

/// i64::MIN is reserved for the long null, so producing it counts as overflow.
fn checked(result: Option<i64>) -> QResult<i64> {
    match result {
        Some(n) if n != i64::MIN => Ok(n),
        _ => Err(QError::Overflow),
    }
}

/// Long null is the i64::MIN sentinel; as a float it is NaN.
fn long_to_float(n: i64) -> f64 {
    if n == i64::MIN {
        f64::NAN
    } else {
        n as f64
    }
}

/// A long null operand yields a long null, otherwise a checked operation.
fn long_op(op: fn(i64, i64) -> Option<i64>, a: i64, b: i64) -> QResult<i64> {
    if a == i64::MIN || b == i64::MIN {
        Ok(i64::MIN)
    } else {
        checked(op(a, b))
    }
}

/// The long and float kernels of a verb. `%` has no long kernel: it is float.
struct Kernel {
    long: Option<fn(i64, i64) -> Option<i64>>,
    float: fn(f64, f64) -> f64,
}

fn kernel(verb: Verb) -> QResult<Kernel> {
    Ok(match verb {
        Verb::Add => Kernel {
            long: Some(i64::checked_add),
            float: |a, b| a + b,
        },
        Verb::Subtract => Kernel {
            long: Some(i64::checked_sub),
            float: |a, b| a - b,
        },
        Verb::Multiply => Kernel {
            long: Some(i64::checked_mul),
            float: |a, b| a * b,
        },
        // IEEE 754: x%0 is inf or NaN, not an error.
        Verb::Divide => Kernel {
            long: None,
            float: |a, b| a / b,
        },
        _ => return Err(QError::Nyi(verb.symbol().into())),
    })
}

/// One numeric operand: a scalar or a slice, as longs or floats (bool is long).
enum Num<'a> {
    Long(Cow<'a, [i64]>),
    Float(Cow<'a, [f64]>),
}

impl Num<'_> {
    fn floats(&self) -> Cow<'_, [f64]> {
        match self {
            Num::Float(f) => Cow::Borrowed(f),
            Num::Long(l) => Cow::Owned(l.iter().map(|&n| long_to_float(n)).collect()),
        }
    }
}

fn num_column(c: &Column) -> QResult<Num<'_>> {
    match c {
        Column::Bool(v) => Ok(Num::Long(Cow::Owned(
            v.iter().map(|&x| i64::from(x)).collect(),
        ))),
        Column::Long(v) => Ok(Num::Long(Cow::Borrowed(v))),
        Column::Float(v) => Ok(Num::Float(Cow::Borrowed(v))),
        Column::Char(_) | Column::Sym(_) => Err(QError::Type),
    }
}

fn num_atom(a: &Atom) -> QResult<Num<'static>> {
    match a {
        Atom::Boolean(x) => Ok(Num::Long(Cow::Owned(vec![i64::from(*x)]))),
        Atom::Integer(x) => Ok(Num::Long(Cow::Owned(vec![*x]))),
        Atom::Float(x) => Ok(Num::Float(Cow::Owned(vec![*x]))),
        _ => Err(QError::Type),
    }
}

/// Pair up two slices, broadcasting a scalar side; two vectors must match in length.
fn zip_with<T: Copy, U>(
    a: &[T],
    a_scalar: bool,
    b: &[T],
    b_scalar: bool,
    mut f: impl FnMut(T, T) -> QResult<U>,
) -> QResult<Vec<U>> {
    let n = match (a_scalar, b_scalar) {
        // two scalars are one element: either length will do
        (true, _) => b.len(),
        (false, true) => a.len(),
        (false, false) if a.len() == b.len() => a.len(),
        (false, false) => return Err(QError::Length),
    };
    (0..n)
        .map(|i| {
            f(
                a[if a_scalar { 0 } else { i }],
                b[if b_scalar { 0 } else { i }],
            )
        })
        .collect()
}

/// The shared kernel: a result column, or a one-element column for two scalars.
fn column_op(verb: Verb, a: &Num, a_scalar: bool, b: &Num, b_scalar: bool) -> QResult<Column> {
    let k = kernel(verb)?;
    if let (Num::Long(x), Num::Long(y), Some(op)) = (a, b, k.long) {
        return zip_with(x, a_scalar, y, b_scalar, |p, q| long_op(op, p, q)).map(Column::Long);
    }
    let (x, y) = (a.floats(), b.floats());
    zip_with(&x, a_scalar, &y, b_scalar, |p, q| Ok((k.float)(p, q))).map(Column::Float)
}

fn atom_op(verb: Verb, a: &Atom, b: &Atom) -> QResult<Atom> {
    let k = kernel(verb)?;
    Ok(match (scalar(a)?, scalar(b)?, k.long) {
        (Scalar::Long(x), Scalar::Long(y), Some(op)) => Atom::Integer(long_op(op, x, y)?),
        (x, y, _) => Atom::Float((k.float)(x.float(), y.float())),
    })
}

#[derive(Clone, Copy)]
enum Scalar {
    Long(i64),
    Float(f64),
}

impl Scalar {
    fn float(self) -> f64 {
        match self {
            Scalar::Long(n) => long_to_float(n),
            Scalar::Float(x) => x,
        }
    }
}

fn scalar(a: &Atom) -> QResult<Scalar> {
    match a {
        Atom::Boolean(x) => Ok(Scalar::Long(i64::from(*x))),
        Atom::Integer(x) => Ok(Scalar::Long(*x)),
        Atom::Float(x) => Ok(Scalar::Float(*x)),
        _ => Err(QError::Type),
    }
}

pub fn dyad(verb: Verb, left: &Value, right: &Value) -> QResult<Value> {
    atomic(verb, left, right, flat)
}

fn flat(verb: Verb, left: Flat, right: Flat) -> QResult<Value> {
    match (left, right) {
        (Flat::Atom(a), Flat::Atom(b)) => atom_op(verb, a, b).map(Value::Atom),
        (Flat::Col(c), Flat::Atom(b)) => {
            column_op(verb, &num_column(c)?, false, &num_atom(b)?, true).map(column_value)
        }
        (Flat::Atom(a), Flat::Col(c)) => {
            column_op(verb, &num_atom(a)?, true, &num_column(c)?, false).map(column_value)
        }
        (Flat::Col(c), Flat::Col(d)) => {
            column_op(verb, &num_column(c)?, false, &num_column(d)?, false).map(column_value)
        }
    }
}

fn column_value(c: Column) -> Value {
    Value::Vector(Rc::new(c))
}

pub fn neg(v: &Value) -> QResult<Value> {
    match v {
        Value::Atom(Atom::Integer(n)) => long_neg(*n).map(|n| Value::Atom(Atom::Integer(n))),
        Value::Atom(Atom::Float(x)) => Ok(Value::Atom(Atom::Float(-x))),
        Value::Vector(c) => match &**c {
            Column::Long(xs) => xs
                .iter()
                .map(|&n| long_neg(n))
                .collect::<QResult<_>>()
                .map(|v| column_value(Column::Long(v))),
            Column::Float(xs) => Ok(column_value(Column::Float(xs.iter().map(|x| -x).collect()))),
            _ => Err(QError::Type),
        },
        Value::List(l) => l
            .iter()
            .map(neg)
            .collect::<QResult<_>>()
            .map(Value::from_items),
        Value::Atom(_) => Err(QError::Type),
    }
}

fn long_neg(n: i64) -> QResult<i64> {
    // Only the null negates to MIN, and it stays null; no other value overflows.
    Ok(if n == i64::MIN { n } else { -n })
}

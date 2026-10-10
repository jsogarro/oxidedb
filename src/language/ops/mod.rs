//! Verb kernels over `Value`s. The interpreter calls these; nothing else
//! knows how a verb treats atoms, vectors and lists.

mod arith;
mod compare;

use crate::error::{QError, QResult};
use crate::language::ast::Verb;
use crate::types::atom::Atom;
use crate::types::column::Column;
use crate::types::value::Value;

/// Apply a binary verb. Verbs without a kernel yet give `'nyi: <verb>`.
pub fn dyad(verb: Verb, left: &Value, right: &Value) -> QResult<Value> {
    match verb {
        Verb::Add | Verb::Subtract | Verb::Multiply | Verb::Divide => {
            arith::dyad(verb, left, right)
        }
        Verb::Equal
        | Verb::NotEqual
        | Verb::Less
        | Verb::LessEqual
        | Verb::Greater
        | Verb::GreaterEqual => compare::dyad(verb, left, right),
        _ => Err(QError::Nyi(verb.symbol().into())),
    }
}

/// Unary minus, atomic like the arithmetic verbs.
pub fn monad_neg(v: &Value) -> QResult<Value> {
    arith::neg(v)
}

/// An operand that is not a general list.
pub(super) enum Flat<'a> {
    Atom(&'a Atom),
    Col(&'a Column),
}

fn elements(c: &Column) -> Vec<Value> {
    (0..c.len()).map(|i| Value::Atom(c.get(i))).collect()
}

/// General path: item against item, renormalised through `from_items`.
fn pairwise(verb: Verb, l: &[Value], r: &[Value], leaf: Leaf) -> QResult<Value> {
    if l.len() != r.len() {
        return Err(QError::Length);
    }
    let out = l.iter().zip(r).map(|(x, y)| atomic(verb, x, y, leaf));
    out.collect::<QResult<_>>().map(Value::from_items)
}

pub(super) type Leaf = fn(Verb, Flat, Flat) -> QResult<Value>;

/// Extend a kernel over general lists: item-wise, recursing into nested lists.
/// `leaf` handles every pair of atoms and vectors.
pub(super) fn atomic(verb: Verb, left: &Value, right: &Value, leaf: Leaf) -> QResult<Value> {
    match (left, right) {
        (Value::Atom(a), Value::Atom(b)) => leaf(verb, Flat::Atom(a), Flat::Atom(b)),
        (Value::Atom(a), Value::Vector(c)) => leaf(verb, Flat::Atom(a), Flat::Col(c)),
        (Value::Vector(c), Value::Atom(b)) => leaf(verb, Flat::Col(c), Flat::Atom(b)),
        (Value::Vector(c), Value::Vector(d)) => leaf(verb, Flat::Col(c), Flat::Col(d)),
        (Value::List(l), Value::List(r)) => pairwise(verb, l, r, leaf),
        (Value::List(l), Value::Vector(c)) => pairwise(verb, l, &elements(c), leaf),
        (Value::Vector(c), Value::List(r)) => pairwise(verb, &elements(c), r, leaf),
        (Value::List(l), Value::Atom(_)) => {
            let out = l.iter().map(|x| atomic(verb, x, right, leaf));
            out.collect::<QResult<_>>().map(Value::from_items)
        }
        (Value::Atom(_), Value::List(r)) => {
            let out = r.iter().map(|y| atomic(verb, left, y, leaf));
            out.collect::<QResult<_>>().map(Value::from_items)
        }
    }
}

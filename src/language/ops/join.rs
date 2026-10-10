//! `x,y`: append. Two simple lists of one type stay simple; anything else is
//! a general list of all the items (no promotion), normalised by `from_items`.
//! An empty operand (typed or `()`) is an identity, whatever its type.

use crate::error::QResult;
use crate::types::column::{checked_len, Column};
use crate::types::value::Value;
use std::rc::Rc;

fn is_empty(v: &Value) -> bool {
    match v {
        Value::Atom(_) => false,
        Value::Vector(c) => c.is_empty(),
        Value::List(l) => l.is_empty(),
    }
}

fn len(v: &Value) -> usize {
    match v {
        Value::Atom(_) => 1,
        Value::Vector(c) => c.len(),
        Value::List(l) => l.len(),
    }
}

/// The items of an operand; a vector or atom splits into atoms.
fn items(v: &Value) -> Vec<Value> {
    match v {
        Value::Atom(_) => vec![v.clone()],
        Value::Vector(c) => (0..c.len()).map(|i| Value::Atom(c.get(i))).collect(),
        Value::List(l) => l.to_vec(),
    }
}

/// An atom or vector as a column; `None` for a list or an atom without one.
fn column(v: &Value) -> Option<Column> {
    match v {
        Value::Atom(a) => Column::from_atoms(std::slice::from_ref(a)),
        Value::Vector(c) => Some((**c).clone()),
        Value::List(_) => None,
    }
}

/// `x` as a list: an atom becomes a one-item list.
fn listify(v: &Value) -> Value {
    match v {
        Value::Atom(_) => Value::from_items(vec![v.clone()]),
        _ => v.clone(),
    }
}

pub fn dyad(left: &Value, right: &Value) -> QResult<Value> {
    checked_len(i64::try_from(len(left) + len(right)).unwrap_or(i64::MAX))?;
    match (is_empty(left), is_empty(right)) {
        // both empty: the right type, but () has none
        (true, true) => Ok(if matches!(right, Value::List(_)) {
            left
        } else {
            right
        }
        .clone()),
        (true, false) => Ok(listify(right)),
        (false, true) => Ok(listify(left)),
        (false, false) => Ok(
            match column(left)
                .zip(column(right))
                .and_then(|(a, b)| a.concat(&b))
            {
                Some(c) => Value::Vector(Rc::new(c)),
                None => Value::from_items([items(left), items(right)].concat()),
            },
        ),
    }
}

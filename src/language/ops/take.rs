//! `n#x`: the first `n` items of `x`, cycling past the end; a negative `n`
//! takes from the end. An atom repeats, and an empty list gives nulls.

use crate::error::{QError, QResult};
use crate::types::atom::Atom;
use crate::types::column::{checked_len, Column};
use crate::types::value::Value;
use std::rc::Rc;

/// The count must be a long atom (a boolean counts as 0 or 1; the long null,
/// like q, is a type error). A list of counts would reshape.
fn count(left: &Value) -> QResult<i64> {
    match left {
        Value::Atom(Atom::Integer(i64::MIN)) => Err(QError::Type),
        Value::Atom(Atom::Integer(n)) => Ok(*n),
        Value::Atom(Atom::Boolean(b)) => Ok(i64::from(*b)),
        Value::Atom(_) => Err(QError::Type),
        Value::Vector(_) | Value::List(_) => Err(QError::Nyi("reshape".into())),
    }
}

pub fn dyad(left: &Value, right: &Value) -> QResult<Value> {
    let n = count(left)?;
    let len = checked_len(n.checked_abs().ok_or(QError::Domain)?)?;
    match right {
        Value::Vector(c) => c.take(n).map(|c| Value::Vector(Rc::new(c))),
        Value::Atom(a) => match Column::from_atoms(std::slice::from_ref(a)) {
            Some(c) => c.take(n).map(|c| Value::Vector(Rc::new(c))),
            // ponytail: temporals have no column, so they repeat as a general list
            None => Ok(Value::List(Rc::new(vec![right.clone(); len]))),
        },
        Value::List(items) if items.is_empty() => {
            Ok(Value::List(Rc::new(vec![Value::List(Rc::default()); len])))
        }
        Value::List(items) => {
            // the item positions are the same cycle a column of 0..len would take
            let positions = Column::Long((0..items.len() as i64).collect()).take(n)?;
            let Column::Long(pos) = positions else {
                unreachable!()
            };
            Ok(Value::from_items(
                pos.iter().map(|&i| items[i as usize].clone()).collect(),
            ))
        }
    }
}

//! `x,y`: append. Two simple lists of one type stay simple; anything else is
//! a general list of all the items (no promotion), normalised by `from_items`.
//! An empty operand (typed or `()`) is an identity, whatever its type.

use crate::error::QResult;
use crate::types::column::{checked_len, Column};
use crate::types::value::Value;
use std::rc::Rc;

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

/// The result length, `'domain` past the element cap.
fn total_len(left: &Value, right: &Value) -> QResult<usize> {
    checked_len(i64::try_from(len(left) + len(right)).unwrap_or(i64::MAX))
}

pub fn dyad(left: &Value, right: &Value) -> QResult<Value> {
    total_len(left, right)?;
    // Nothing to normalise from: the result keeps the right type, but () has none.
    if len(left) + len(right) == 0 {
        return Ok(if matches!(right, Value::List(_)) {
            left
        } else {
            right
        }
        .clone());
    }
    Ok(
        match column(left)
            .zip(column(right))
            .and_then(|(a, b)| a.concat(&b))
        {
            Some(c) => Value::Vector(Rc::new(c)),
            None => Value::from_items([items(left), items(right)].concat()),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::QError;
    use crate::types::atom::Atom;
    use crate::types::column::MAX_ELEMS;

    fn general(n: usize) -> Value {
        Value::List(Rc::new(vec![Value::Atom(Atom::Boolean(true)); n]))
    }

    #[test]
    fn join_len_counts_general_list_items() {
        let one = Value::Atom(Atom::Integer(1));
        assert_eq!(len(&general(3)), 3);
        assert_eq!(total_len(&general(3), &one), Ok(4));
        assert_eq!(total_len(&one, &general(3)), Ok(4));
    }

    #[test]
    fn join_cap_applies_to_general_lists() {
        // building a list at the real cap would cost hundreds of MB, so the
        // pure length check is tested at the cap and one real join below it
        let one = Value::Atom(Atom::Integer(1));
        let empty = Value::List(Rc::default());
        assert_eq!(total_len(&general(MAX_ELEMS), &empty), Ok(MAX_ELEMS));
        assert_eq!(total_len(&general(MAX_ELEMS), &one), Err(QError::Domain));
        assert_eq!(total_len(&one, &general(MAX_ELEMS)), Err(QError::Domain));
        assert_eq!(dyad(&general(4), &empty).map(|v| len(&v)), Ok(4));
    }
}

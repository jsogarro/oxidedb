//! Applying a value to arguments: `v i`, `v[i]`. A vector or general list is
//! indexed; an atom cannot be applied (q treats an integer atom applied to an
//! argument as a file handle, which O does not imitate).

use crate::error::{QError, QResult};
use crate::types::atom::Atom;
use crate::types::column::Column;
use crate::types::value::Value;
use std::rc::Rc;

/// `f[args]`. No arguments gives `f` back; a vector takes one index.
pub fn apply(f: &Value, args: &[Value]) -> QResult<Value> {
    match (f, args) {
        (Value::Atom(_), _) => Err(QError::Type),
        (_, []) => Ok(f.clone()),
        (_, [i]) => index(f, i),
        (Value::Vector(_), _) => Err(QError::Rank),
        (Value::List(_), _) => Err(QError::Nyi("depth indexing".into())),
    }
}

/// An index atom as a position; negative and out-of-range positions are
/// handled by the caller (they give nulls). Booleans count as 0 and 1, as in q.
fn position(a: &Atom) -> QResult<i64> {
    match a {
        Atom::Integer(n) => Ok(*n),
        Atom::Boolean(b) => Ok(i64::from(*b)),
        _ => Err(QError::Type),
    }
}

fn index(f: &Value, i: &Value) -> QResult<Value> {
    match i {
        Value::Atom(a) => {
            let n = usize::try_from(position(a)?).unwrap_or(usize::MAX);
            Ok(match f {
                Value::Vector(c) => Value::Atom(c.get(n)),
                // ponytail: q gives the null of the first item's type; a long null until lists can be built
                Value::List(items) => items
                    .get(n)
                    .cloned()
                    .unwrap_or(Value::Atom(Atom::Integer(i64::MIN))),
                Value::Atom(_) => unreachable!("atoms are rejected before indexing"),
            })
        }
        Value::Vector(c) => {
            let idx: Vec<i64> = match &**c {
                Column::Long(v) => v.clone(),
                Column::Bool(v) => v.iter().map(|&b| i64::from(b)).collect(),
                _ => return Err(QError::Type),
            };
            Ok(match f {
                Value::Vector(c) => Value::Vector(Rc::new(c.index(&idx))),
                _ => {
                    let items = idx
                        .iter()
                        .map(|&k| index(f, &Value::Atom(Atom::Integer(k))));
                    Value::from_items(items.collect::<QResult<_>>()?)
                }
            })
        }
        Value::List(items) => {
            let out = items.iter().map(|x| index(f, x));
            Ok(Value::from_items(out.collect::<QResult<_>>()?))
        }
    }
}

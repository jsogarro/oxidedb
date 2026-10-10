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

/// `target[idx]:val`, in place (copy-on-write when the vector is shared). Checked in q's order,
/// and nothing changes on an error: the index must be long or boolean (`'type`), every position
/// in range (`'length`, where a read gives a null), the value's count must fit (`'length`),
/// and for a vector its type must match exactly (`'type`: no promotion). An atom cannot be
/// amended. A general list takes any value; one that ends up with all items of one atom type
/// collapses to a vector, as in q.
pub fn amend(target: &mut Value, idx: &Value, val: &Value) -> QResult<()> {
    let len = match target {
        Value::Atom(_) => return Err(QError::Type),
        Value::Vector(c) => c.len(),
        Value::List(items) => items.len(),
    };
    let (positions, scalar) = match idx {
        Value::Atom(a) => (vec![position(a)?], true),
        Value::Vector(c) => match &**c {
            Column::Long(v) => (v.clone(), false),
            Column::Bool(v) => (v.iter().map(|&b| i64::from(b)).collect(), false),
            _ => return Err(QError::Type),
        },
        // the empty general list is an empty index
        Value::List(items) if items.is_empty() => return Ok(()),
        Value::List(_) => return Err(QError::Type),
    };
    let pos = positions
        .iter()
        .map(|&p| usize::try_from(p).ok().filter(|&p| p < len))
        .collect::<Option<Vec<usize>>>()
        .ok_or(QError::Length)?;
    // One item per position, or a single item broadcast.
    let count_fits = |n: usize| scalar || n == pos.len();
    match target {
        Value::Vector(col) => {
            let src = match val {
                Value::Atom(a) => {
                    Column::from_atoms(std::slice::from_ref(a)).ok_or(QError::Type)?
                }
                _ if scalar => return Err(QError::Type),
                Value::Vector(c) if count_fits(c.len()) => (**c).clone(),
                Value::List(items) if count_fits(items.len()) => {
                    let atoms: Option<Vec<Atom>> = items
                        .iter()
                        .map(|x| match x {
                            Value::Atom(a) => Some(a.clone()),
                            _ => None,
                        })
                        .collect();
                    atoms
                        .as_deref()
                        .and_then(Column::from_atoms)
                        .ok_or(QError::Type)?
                }
                _ => return Err(QError::Length),
            };
            Rc::make_mut(col).assign(&pos, &src)
        }
        Value::List(items) => {
            let new: Vec<Value> = match val {
                Value::Atom(_) => vec![val.clone()],
                _ if scalar => vec![val.clone()],
                Value::Vector(c) if count_fits(c.len()) => {
                    (0..c.len()).map(|k| Value::Atom(c.get(k))).collect()
                }
                Value::List(l) if count_fits(l.len()) => l.to_vec(),
                _ => return Err(QError::Length),
            };
            let list = Rc::make_mut(items);
            for (k, &p) in pos.iter().enumerate() {
                list[p] = if new.len() == 1 {
                    new[0].clone()
                } else {
                    new[k].clone()
                };
            }
            let all = std::mem::take(list);
            *target = Value::from_items(all);
            Ok(())
        }
        Value::Atom(_) => unreachable!("atoms are rejected above"),
    }
}

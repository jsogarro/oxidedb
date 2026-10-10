//! Named builtin functions (`til`, `count`). The interpreter resolves a name
//! to a builtin after variables, then calls it with already-evaluated args.

use crate::error::{QError, QResult};
use crate::types::atom::Atom;
use crate::types::column::Column;
use crate::types::value::Value;
use std::rc::Rc;

/// Largest `til n` allowed, so a typo cannot exhaust memory.
pub const MAX_TIL: i64 = 100_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Builtin {
    Til,
    Count,
}

pub fn lookup(name: &str) -> Option<Builtin> {
    match name {
        "til" => Some(Builtin::Til),
        "count" => Some(Builtin::Count),
        _ => None,
    }
}

/// Call the builtin `name`. Every current builtin takes exactly one argument
/// (`'rank` otherwise); an unknown name is `'name (Undefined variable)`.
pub fn call(name: &str, args: &[Value]) -> QResult<Value> {
    let b = lookup(name).ok_or_else(|| QError::Undefined(name.to_string()))?;
    let [x] = args else {
        return Err(QError::Rank);
    };
    match b {
        Builtin::Til => til(x),
        Builtin::Count => Ok(count(x)),
    }
}

fn til(x: &Value) -> QResult<Value> {
    let Value::Atom(Atom::Integer(n)) = x else {
        return Err(QError::Type);
    };
    if !(0..=MAX_TIL).contains(n) {
        return Err(QError::Domain);
    }
    Ok(Value::Vector(Rc::new(Column::Long((0..*n).collect()))))
}

fn count(x: &Value) -> Value {
    let n = match x {
        Value::Atom(_) => 1,
        Value::Vector(c) => c.len(),
        Value::List(items) => items.len(),
    };
    Value::Atom(Atom::Integer(n as i64))
}

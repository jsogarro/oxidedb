//! Named builtin functions (`til`, `count`). The interpreter resolves a name
//! to a builtin after variables, then calls it with already-evaluated args.

use crate::error::{QError, QResult};
use crate::types::atom::Atom;
use crate::types::column::{checked_len, Column};
use crate::types::value::Value;
use std::rc::Rc;

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

impl Builtin {
    pub fn name(self) -> &'static str {
        match self {
            Builtin::Til => "til",
            Builtin::Count => "count",
        }
    }

    pub fn arity(self) -> usize {
        match self {
            Builtin::Til | Builtin::Count => 1,
        }
    }

    /// `'rank` when `args` does not match the arity.
    pub fn call(self, args: &[Value]) -> QResult<Value> {
        if args.len() != self.arity() {
            return Err(QError::Rank);
        }
        match self {
            Builtin::Til => til(&args[0]),
            Builtin::Count => Ok(count(&args[0])),
        }
    }
}

/// Call the builtin `name`; an unknown name is `'name (Undefined variable)`.
pub fn call(name: &str, args: &[Value]) -> QResult<Value> {
    lookup(name)
        .ok_or_else(|| QError::Undefined(name.to_string()))?
        .call(args)
}

fn til(x: &Value) -> QResult<Value> {
    let Value::Atom(Atom::Integer(n)) = x else {
        return Err(QError::Type);
    };
    let n = checked_len(*n)? as i64;
    Ok(Value::Vector(Rc::new(Column::Long((0..n).collect()))))
}

fn count(x: &Value) -> Value {
    let n = match x {
        Value::Atom(_) => 1,
        Value::Vector(c) => c.len(),
        Value::List(items) => items.len(),
    };
    Value::Atom(Atom::Integer(n as i64))
}

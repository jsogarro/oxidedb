//! Named builtin functions (`til`, `count`, `neg`, `enlist`). The interpreter resolves a name
//! to a builtin after variables, then calls it with already-evaluated args.

use crate::error::{QError, QResult};
use crate::language::ops;
use crate::types::atom::Atom;
use crate::types::column::{checked_len, Column};
use crate::types::value::Value;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Builtin {
    Til,
    Count,
    Neg,
    Enlist,
}

pub fn lookup(name: &str) -> Option<Builtin> {
    match name {
        "til" => Some(Builtin::Til),
        "count" => Some(Builtin::Count),
        "neg" => Some(Builtin::Neg),
        "enlist" => Some(Builtin::Enlist),
        _ => None,
    }
}

impl Builtin {
    pub fn name(self) -> &'static str {
        match self {
            Builtin::Til => "til",
            Builtin::Count => "count",
            Builtin::Neg => "neg",
            Builtin::Enlist => "enlist",
        }
    }

    pub fn arity(self) -> usize {
        match self {
            Builtin::Til | Builtin::Count | Builtin::Neg | Builtin::Enlist => 1,
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
            Builtin::Neg => ops::monad_neg(&args[0]),
            // an atom becomes a one-item vector, anything else a one-item general list
            Builtin::Enlist => checked(Value::from_items(vec![args[0].clone()])),
        }
    }
}

fn checked(v: Value) -> QResult<Value> {
    v.check_nesting(0)?;
    Ok(v)
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

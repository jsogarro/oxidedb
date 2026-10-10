//! Verb kernels over `Value`s. The interpreter calls these; nothing else
//! knows how a verb treats atoms, vectors and lists.

mod arith;

use crate::error::{QError, QResult};
use crate::language::ast::Verb;
use crate::types::value::Value;

/// Apply a binary verb. Verbs without a kernel yet give `'nyi: <verb>`.
pub fn dyad(verb: Verb, left: &Value, right: &Value) -> QResult<Value> {
    match verb {
        Verb::Add | Verb::Subtract | Verb::Multiply | Verb::Divide => {
            arith::dyad(verb, left, right)
        }
        _ => Err(QError::Nyi(verb.symbol().into())),
    }
}

/// Unary minus, atomic like the arithmetic verbs.
pub fn monad_neg(v: &Value) -> QResult<Value> {
    arith::neg(v)
}

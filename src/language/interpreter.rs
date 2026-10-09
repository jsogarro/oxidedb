use crate::language::ast::{BinaryOperator, Expr, UnaryOperator};
use crate::types::atom::Atom;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// i64::MIN is reserved for the long null, so producing it counts as overflow.
fn checked(result: Option<i64>) -> Result<Atom> {
    match result {
        Some(n) if n != i64::MIN => Ok(Atom::Integer(n)),
        _ => Err(anyhow!("Integer overflow")),
    }
}

pub struct Interpreter {
    variables: HashMap<String, Atom>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn evaluate(&mut self, expr: Expr) -> Result<Atom> {
        match expr {
            Expr::Atom(atom) => Ok(atom),
            Expr::Symbol(name) => self
                .variables
                .get(&name)
                .cloned()
                .ok_or_else(|| anyhow!("Undefined variable: {}", name)),
            Expr::BinaryOp {
                left,
                operator,
                right,
            } => {
                // q evaluates right to left, including side effects.
                let right_val = self.evaluate(*right)?;
                let left_val = self.evaluate(*left)?;
                self.apply_binary_op(&left_val, &operator, &right_val)
            }
            Expr::UnaryOp { operator, operand } => {
                let val = self.evaluate(*operand)?;
                self.apply_unary_op(&operator, &val)
            }
            Expr::Assignment { name, value } => {
                let val = self.evaluate(*value)?;
                self.variables.insert(name, val.clone());
                Ok(val)
            }
        }
    }

    fn apply_binary_op(&self, left: &Atom, op: &BinaryOperator, right: &Atom) -> Result<Atom> {
        match (left, op, right) {
            // Integer arithmetic
            (Atom::Integer(a), BinaryOperator::Add, Atom::Integer(b)) => checked(a.checked_add(*b)),
            (Atom::Integer(a), BinaryOperator::Subtract, Atom::Integer(b)) => {
                checked(a.checked_sub(*b))
            }
            (Atom::Integer(a), BinaryOperator::Multiply, Atom::Integer(b)) => {
                checked(a.checked_mul(*b))
            }
            // `%` is always float division, as in q.
            (Atom::Integer(a), BinaryOperator::Divide, Atom::Integer(b)) => {
                Ok(Atom::Float(*a as f64 / *b as f64))
            }

            // Float arithmetic (with type promotion)
            (Atom::Float(a), BinaryOperator::Add, Atom::Float(b)) => Ok(Atom::Float(a + b)),
            (Atom::Float(a), BinaryOperator::Subtract, Atom::Float(b)) => Ok(Atom::Float(a - b)),
            (Atom::Float(a), BinaryOperator::Multiply, Atom::Float(b)) => Ok(Atom::Float(a * b)),
            // IEEE 754: x%0 is inf or NaN, not an error.
            (Atom::Float(a), BinaryOperator::Divide, Atom::Float(b)) => Ok(Atom::Float(a / b)),

            // Mixed integer/float arithmetic (promote to float)
            (Atom::Integer(a), op, Atom::Float(b)) => {
                self.apply_binary_op(&Atom::Float(*a as f64), op, &Atom::Float(*b))
            }
            (Atom::Float(a), op, Atom::Integer(b)) => {
                self.apply_binary_op(&Atom::Float(*a), op, &Atom::Float(*b as f64))
            }

            _ => Err(anyhow!(
                "Invalid binary operation: {:?} {:?} {:?}",
                left,
                op,
                right
            )),
        }
    }

    fn apply_unary_op(&self, op: &UnaryOperator, operand: &Atom) -> Result<Atom> {
        match (op, operand) {
            (UnaryOperator::Negate, Atom::Integer(n)) => checked(n.checked_neg()),
            (UnaryOperator::Negate, Atom::Float(f)) => Ok(Atom::Float(-f)),
            _ => Err(anyhow!("Invalid unary operation: {:?} {:?}", op, operand)),
        }
    }
}

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use crate::language::ast::{Expr, BinaryOperator, UnaryOperator};
use crate::types::atom::Atom;

pub struct Interpreter {
    variables: HashMap<String, Atom>,
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
            Expr::Symbol(name) => {
                self.variables.get(&name)
                    .cloned()
                    .ok_or_else(|| anyhow!("Undefined variable: {}", name))
            }
            Expr::BinaryOp { left, operator, right } => {
                let left_val = self.evaluate(*left)?;
                let right_val = self.evaluate(*right)?;
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
            (Atom::Integer(a), BinaryOperator::Add, Atom::Integer(b)) => Ok(Atom::Integer(a + b)),
            (Atom::Integer(a), BinaryOperator::Subtract, Atom::Integer(b)) => Ok(Atom::Integer(a - b)),
            (Atom::Integer(a), BinaryOperator::Multiply, Atom::Integer(b)) => Ok(Atom::Integer(a * b)),
            (Atom::Integer(a), BinaryOperator::Divide, Atom::Integer(b)) => {
                if *b == 0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(Atom::Integer(a / b))
                }
            }
            
            // Float arithmetic (with type promotion)
            (Atom::Float(a), BinaryOperator::Add, Atom::Float(b)) => Ok(Atom::Float(a + b)),
            (Atom::Float(a), BinaryOperator::Subtract, Atom::Float(b)) => Ok(Atom::Float(a - b)),
            (Atom::Float(a), BinaryOperator::Multiply, Atom::Float(b)) => Ok(Atom::Float(a * b)),
            (Atom::Float(a), BinaryOperator::Divide, Atom::Float(b)) => {
                if *b == 0.0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(Atom::Float(a / b))
                }
            }
            
            // Mixed integer/float arithmetic (promote to float)
            (Atom::Integer(a), op, Atom::Float(b)) => {
                self.apply_binary_op(&Atom::Float(*a as f64), op, &Atom::Float(*b))
            }
            (Atom::Float(a), op, Atom::Integer(b)) => {
                self.apply_binary_op(&Atom::Float(*a), op, &Atom::Float(*b as f64))
            }
            
            _ => Err(anyhow!("Invalid binary operation: {:?} {:?} {:?}", left, op, right)),
        }
    }

    fn apply_unary_op(&self, op: &UnaryOperator, operand: &Atom) -> Result<Atom> {
        match (op, operand) {
            (UnaryOperator::Negate, Atom::Integer(n)) => Ok(Atom::Integer(-n)),
            (UnaryOperator::Negate, Atom::Float(f)) => Ok(Atom::Float(-f)),
            _ => Err(anyhow!("Invalid unary operation: {:?} {:?}", op, operand)),
        }
    }
}
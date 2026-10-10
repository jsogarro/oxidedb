use crate::error::QError;
use crate::language::ast::{BinaryOperator, Expr, UnaryOperator};
use crate::language::{
    lexer::{Lexer, Token},
    parser::Parser,
};
use crate::types::atom::Atom;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, QError>;

/// i64::MIN is reserved for the long null, so producing it counts as overflow.
fn checked(result: Option<i64>) -> Result<Atom> {
    match result {
        Some(n) if n != i64::MIN => Ok(Atom::Integer(n)),
        _ => Err(QError::Overflow),
    }
}

/// Long null is the i64::MIN sentinel; as a float it is NaN.
fn long_to_float(n: i64) -> f64 {
    if n == i64::MIN {
        f64::NAN
    } else {
        n as f64
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

    /// Lex, parse and evaluate one line. `None` means the line had no tokens
    /// (e.g. blank or comment-only).
    pub fn eval_line(&mut self, input: &str) -> Result<Option<Atom>> {
        let tokens = Lexer::new(input).tokenize()?;
        if tokens == [Token::Eof] {
            return Ok(None);
        }
        let ast = Parser::new(tokens).parse()?;
        self.evaluate(ast).map(Some)
    }

    pub fn evaluate(&mut self, expr: Expr) -> Result<Atom> {
        match expr {
            Expr::Atom(atom) => Ok(atom),
            Expr::Symbol(name) => self
                .variables
                .get(&name)
                .cloned()
                .ok_or(QError::Undefined(name)),
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
            // A long null operand yields a long null (`%` is float, handled below).
            (
                Atom::Integer(a),
                BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply,
                Atom::Integer(b),
            ) if *a == i64::MIN || *b == i64::MIN => Ok(Atom::Integer(i64::MIN)),
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
                Ok(Atom::Float(long_to_float(*a) / long_to_float(*b)))
            }

            // Float arithmetic (with type promotion)
            (Atom::Float(a), BinaryOperator::Add, Atom::Float(b)) => Ok(Atom::Float(a + b)),
            (Atom::Float(a), BinaryOperator::Subtract, Atom::Float(b)) => Ok(Atom::Float(a - b)),
            (Atom::Float(a), BinaryOperator::Multiply, Atom::Float(b)) => Ok(Atom::Float(a * b)),
            // IEEE 754: x%0 is inf or NaN, not an error.
            (Atom::Float(a), BinaryOperator::Divide, Atom::Float(b)) => Ok(Atom::Float(a / b)),

            // Mixed integer/float arithmetic (promote to float)
            (Atom::Integer(a), op, Atom::Float(b)) => {
                self.apply_binary_op(&Atom::Float(long_to_float(*a)), op, &Atom::Float(*b))
            }
            (Atom::Float(a), op, Atom::Integer(b)) => {
                self.apply_binary_op(&Atom::Float(*a), op, &Atom::Float(long_to_float(*b)))
            }

            _ => Err(QError::Type),
        }
    }

    fn apply_unary_op(&self, op: &UnaryOperator, operand: &Atom) -> Result<Atom> {
        match (op, operand) {
            (UnaryOperator::Negate, Atom::Integer(i64::MIN)) => Ok(Atom::Integer(i64::MIN)),
            (UnaryOperator::Negate, Atom::Integer(n)) => checked(n.checked_neg()),
            (UnaryOperator::Negate, Atom::Float(f)) => Ok(Atom::Float(-f)),
            _ => Err(QError::Type),
        }
    }
}

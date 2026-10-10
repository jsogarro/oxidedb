use crate::error::{QError, QResult};
use crate::language::ast::{Expr, UnaryOperator, Verb};
use crate::language::{
    lexer::{Lexer, Token},
    parser::Parser,
};
use crate::types::atom::Atom;
use crate::types::value::Value;
use std::collections::HashMap;

/// i64::MIN is reserved for the long null, so producing it counts as overflow.
fn checked(result: Option<i64>) -> QResult<Atom> {
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
    variables: HashMap<String, Value>,
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
    pub fn eval_line(&mut self, input: &str) -> QResult<Option<Value>> {
        let tokens = Lexer::new(input).tokenize()?;
        if tokens == [Token::Eof] {
            return Ok(None);
        }
        let ast = Parser::new(tokens).parse()?;
        self.evaluate(ast).map(Some)
    }

    /// Bind `name` to `value`, as `name:value` would.
    pub fn set(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn evaluate(&mut self, expr: Expr) -> QResult<Value> {
        match expr {
            Expr::Atom(atom) => Ok(Value::Atom(atom)),
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
                match (&left_val, &right_val) {
                    (Value::Atom(l), Value::Atom(r)) => {
                        self.apply_binary_op(l, &operator, r).map(Value::Atom)
                    }
                    // ponytail: vector arithmetic arrives with the kernel (#47).
                    _ => Err(QError::Nyi("vector arithmetic".into())),
                }
            }
            Expr::UnaryOp { operator, operand } => {
                let val = self.evaluate(*operand)?;
                match &val {
                    Value::Atom(a) => self.apply_unary_op(&operator, a).map(Value::Atom),
                    _ => Err(QError::Nyi("vector arithmetic".into())),
                }
            }
            Expr::Assignment { name, value } => {
                let val = self.evaluate(*value)?;
                self.variables.insert(name, val.clone());
                Ok(val)
            }
        }
    }

    fn apply_binary_op(&self, left: &Atom, op: &Verb, right: &Atom) -> QResult<Atom> {
        match (left, op, right) {
            (
                _,
                Verb::Equal
                | Verb::Less
                | Verb::Greater
                | Verb::NotEqual
                | Verb::LessEqual
                | Verb::GreaterEqual
                | Verb::Take
                | Verb::Join
                | Verb::Key,
                _,
            ) => Err(QError::Nyi(op.symbol().into())),
            // A long null operand yields a long null (`%` is float, handled below).
            (Atom::Integer(a), Verb::Add | Verb::Subtract | Verb::Multiply, Atom::Integer(b))
                if *a == i64::MIN || *b == i64::MIN =>
            {
                Ok(Atom::Integer(i64::MIN))
            }
            // Integer arithmetic
            (Atom::Integer(a), Verb::Add, Atom::Integer(b)) => checked(a.checked_add(*b)),
            (Atom::Integer(a), Verb::Subtract, Atom::Integer(b)) => checked(a.checked_sub(*b)),
            (Atom::Integer(a), Verb::Multiply, Atom::Integer(b)) => checked(a.checked_mul(*b)),
            // `%` is always float division, as in q.
            (Atom::Integer(a), Verb::Divide, Atom::Integer(b)) => {
                Ok(Atom::Float(long_to_float(*a) / long_to_float(*b)))
            }

            // Float arithmetic (with type promotion)
            (Atom::Float(a), Verb::Add, Atom::Float(b)) => Ok(Atom::Float(a + b)),
            (Atom::Float(a), Verb::Subtract, Atom::Float(b)) => Ok(Atom::Float(a - b)),
            (Atom::Float(a), Verb::Multiply, Atom::Float(b)) => Ok(Atom::Float(a * b)),
            // IEEE 754: x%0 is inf or NaN, not an error.
            (Atom::Float(a), Verb::Divide, Atom::Float(b)) => Ok(Atom::Float(a / b)),

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

    fn apply_unary_op(&self, op: &UnaryOperator, operand: &Atom) -> QResult<Atom> {
        match (op, operand) {
            (UnaryOperator::Negate, Atom::Integer(i64::MIN)) => Ok(Atom::Integer(i64::MIN)),
            (UnaryOperator::Negate, Atom::Integer(n)) => checked(n.checked_neg()),
            (UnaryOperator::Negate, Atom::Float(f)) => Ok(Atom::Float(-f)),
            _ => Err(QError::Type),
        }
    }
}

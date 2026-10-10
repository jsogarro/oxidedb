use crate::error::{QError, QResult};
use crate::language::ast::{Expr, UnaryOperator};
use crate::language::ops;
use crate::language::{
    lexer::{Lexer, Token},
    parser::Parser,
};
use crate::types::value::Value;
use std::collections::HashMap;

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
            Expr::Lit(value) => Ok(value),
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
                ops::dyad(operator, &left_val, &right_val)
            }
            Expr::UnaryOp { operator, operand } => {
                let val = self.evaluate(*operand)?;
                match operator {
                    UnaryOperator::Negate => ops::monad_neg(&val),
                }
            }
            Expr::Assignment { name, value } => {
                let val = self.evaluate(*value)?;
                self.variables.insert(name, val.clone());
                Ok(val)
            }
        }
    }
}

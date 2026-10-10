use crate::error::{QError, QResult};
use crate::language::ast::{Expr, UnaryOperator};
use crate::language::{apply, builtins, ops};
use crate::language::{
    lexer::{Lexer, Token},
    parser::Parser,
};
use crate::types::column::Column;
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
            Expr::Symbol(name) => match self.variables.get(&name) {
                Some(value) => Ok(value.clone()),
                // ponytail: no function values yet, so a bare builtin cannot be returned
                None if builtins::lookup(&name).is_some() => {
                    Err(QError::Nyi(format!("{name} as a value")))
                }
                None => Err(QError::Undefined(name)),
            },
            Expr::Apply { func, args } => self.evaluate_apply(*func, args),
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
            Expr::List(items) => {
                // right to left, like every other construct
                let mut vals = Vec::with_capacity(items.len());
                for item in items.into_iter().rev() {
                    vals.push(self.evaluate(item)?);
                }
                vals.reverse();
                {
                    let list = Value::from_items(vals);
                    list.check_nesting(0)?;
                    Ok(list)
                }
            }
            Expr::Assignment { name, value } => {
                let val = self.evaluate(*value)?;
                self.variables.insert(name, val.clone());
                Ok(val)
            }
            Expr::IndexAssignment { name, index, value } => {
                self.evaluate_index_assignment(&name, index.map(|i| *i), *value)
            }
        }
    }

    /// `name[index]:value`. Out of line like `evaluate_apply`. The value is evaluated first,
    /// then the index, as q does; only then is the name read.
    #[inline(never)]
    fn evaluate_index_assignment(
        &mut self,
        name: &str,
        index: Option<Expr>,
        value: Expr,
    ) -> QResult<Value> {
        let value = self.evaluate(value)?;
        let index = index.map(|i| self.evaluate(i)).transpose()?;
        // Deliberately not q, which reads an undefined name as `()` ('length): the error a read gives.
        let target = self
            .variables
            .get_mut(name)
            .ok_or_else(|| QError::Undefined(name.to_string()))?;
        // `v[]:x` assigns every item
        let index = match index {
            Some(index) => index,
            None => match target {
                Value::Atom(_) => return Err(QError::Type),
                Value::Vector(c) => til(c.len()),
                Value::List(l) => til(l.len()),
            },
        };
        apply::amend(target, &index, &value)?;
        // The statement's value is what the index now reads (duplicates: the last one won).
        apply::apply(target, &[index])
    }

    /// Kept out of line so `evaluate`'s own stack frame, which every nesting level pays for,
    /// does not grow with this arm's locals.
    #[inline(never)]
    fn evaluate_apply(&mut self, func: Expr, args: Vec<Expr>) -> QResult<Value> {
        // Arguments right to left, then the function, so a side effect in an
        // argument happens before the function or vector is read.
        let mut vals = Vec::with_capacity(args.len());
        for arg in args.into_iter().rev() {
            vals.push(self.evaluate(arg)?);
        }
        vals.reverse();
        if let Expr::Symbol(name) = &func {
            // A variable wins over a builtin of the same name.
            if !self.variables.contains_key(name) {
                return builtins::call(name, &vals);
            }
        }
        let f = self.evaluate(func)?;
        apply::apply(&f, &vals)
    }
}

/// `0 1 .. n-1` as a long vector.
fn til(n: usize) -> Value {
    Value::Vector(std::rc::Rc::new(Column::Long((0..n as i64).collect())))
}

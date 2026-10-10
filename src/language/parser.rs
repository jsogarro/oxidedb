use crate::error::{QError, QResult};
use crate::language::{
    ast::{Expr, UnaryOperator, Verb},
    lexer::Token,
};
use crate::types::atom::Atom;
use crate::types::column::Column;
use crate::types::sym::Sym;
use crate::types::value::Value;
use std::rc::Rc;

/// The not-yet-implemented error for a token the parser cannot handle yet
/// (adverbs, punctuation, a verb used monadically); `None` for any other token.
fn nyi_token(token: &Token) -> Option<QError> {
    let detail = match token {
        Token::Over | Token::Scan => format!("adverb '{token}'"),
        Token::Quote => "adverb ' (each)".to_string(),
        Token::EachPrior => "adverb ': (each-prior)".to_string(),
        Token::EachRight => "adverb /: (each-right)".to_string(),
        Token::EachLeft => "adverb \\: (each-left)".to_string(),
        Token::Equal
        | Token::Less
        | Token::Greater
        | Token::NotEqual
        | Token::LessEqual
        | Token::GreaterEqual
        | Token::Hash
        | Token::Comma
        | Token::Bang
        | Token::Dollar
        | Token::At
        | Token::LeftBrace
        | Token::RightBrace
        | Token::DoubleColon => token.to_string(),
        _ => return None,
    };
    Some(QError::Nyi(detail))
}

/// The binary verb a token spells, if any.
fn verb_of(token: &Token) -> Option<Verb> {
    Some(match token {
        Token::Plus => Verb::Add,
        Token::Minus => Verb::Subtract,
        Token::Multiply => Verb::Multiply,
        Token::Divide => Verb::Divide,
        Token::Equal => Verb::Equal,
        Token::Less => Verb::Less,
        Token::Greater => Verb::Greater,
        Token::NotEqual => Verb::NotEqual,
        Token::LessEqual => Verb::LessEqual,
        Token::GreaterEqual => Verb::GreaterEqual,
        Token::Hash => Verb::Take,
        Token::Comma => Verb::Join,
        Token::Bang => Verb::Key,
        _ => return None,
    })
}

fn vector(column: Column) -> Expr {
    Expr::Lit(Value::Vector(Rc::new(column)))
}

/// Tokens that begin a noun (a literal, a name or a parenthesised expression).
fn starts_noun(token: &Token) -> bool {
    matches!(
        token,
        Token::Integer(_)
            | Token::Float(_)
            | Token::Boolean(_)
            | Token::Character(_)
            | Token::Symbol(_)
            | Token::Sym(_)
            | Token::SymList(_)
            | Token::Str(_)
            | Token::BoolList(_)
            | Token::LeftParen
    )
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    depth: usize,
}

/// Maximum nesting of parentheses / monadic minus.
const MAX_DEPTH: usize = 128;
/// Maximum operators in one flat chain (`1+1+...`); evaluate and drop recurse over it.
const MAX_CHAIN: usize = 2_000;

impl Parser {
    pub fn new(mut tokens: Vec<Token>) -> Self {
        if !matches!(tokens.last(), Some(Token::Eof)) {
            tokens.push(Token::Eof);
        }
        Self {
            tokens,
            current: 0,
            depth: 0,
        }
    }

    pub fn parse(&mut self) -> QResult<Expr> {
        let expr = self.expression()?;
        if !self.is_at_end() {
            if let Some(err) = nyi_token(self.peek()) {
                return Err(err);
            }
            return Err(QError::parse(format!(
                "unexpected {} after expression",
                self.peek()
            )));
        }
        Ok(expr)
    }

    fn expression(&mut self) -> QResult<Expr> {
        if self.depth >= MAX_DEPTH {
            return Err(QError::parse("expression nested too deeply"));
        }
        self.depth += 1;
        let result = self.assignment();
        self.depth -= 1;
        result
    }

    fn assignment(&mut self) -> QResult<Expr> {
        let expr = self.binary_expression()?;

        // Check if this is an assignment (symbol followed by colon)
        if let Expr::Symbol(name) = &expr {
            if self.match_tokens(&[Token::Colon]) {
                let value = self.expression()?;
                return Ok(Expr::Assignment {
                    name: name.clone(),
                    value: Box::new(value),
                });
            }
        }

        Ok(expr)
    }

    fn binary_expression(&mut self) -> QResult<Expr> {
        // Parsed iteratively so a long flat chain does not consume parser depth;
        // folded from the right into the same right-associative AST.
        let mut operands = Vec::new();
        let mut operators = Vec::new();
        loop {
            let mut operand = self.unary()?;
            // An assignment in operand position (`x+y:2`) takes the rest of the input.
            if let (false, Expr::Symbol(name)) = (operands.is_empty(), &operand) {
                if self.match_tokens(&[Token::Colon]) {
                    let name = name.clone();
                    let value = self.expression()?;
                    operand = Expr::Assignment {
                        name,
                        value: Box::new(value),
                    };
                }
            }
            // A noun straight after a noun is application (`x 1`, `1 "a"`), not built yet.
            if starts_noun(self.peek()) {
                return Err(QError::Nyi("application".into()));
            }
            operands.push(operand);
            let Some(verb) = verb_of(self.peek()) else {
                break;
            };
            self.advance();
            operators.push(verb);
            // The AST is still a right-nested tree, which evaluate and drop recurse over.
            if operators.len() >= MAX_CHAIN {
                return Err(QError::parse("expression too long"));
            }
        }
        let mut right = operands.pop().expect("at least one operand");
        while let Some(operator) = operators.pop() {
            let left = operands.pop().expect("operand per operator");
            right = Expr::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        Ok(right)
    }

    fn unary(&mut self) -> QResult<Expr> {
        if self.match_tokens(&[Token::Minus]) {
            // Monadic minus takes its whole right side (O's rule; q uses `neg`): -x+3 is -(x+3).
            let expr = self.expression()?;
            return Ok(Expr::UnaryOp {
                operator: UnaryOperator::Negate,
                operand: Box::new(expr),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> QResult<Expr> {
        if self.is_at_end() {
            return Err(QError::parse("unexpected end of input"));
        }
        if matches!(self.peek(), Token::Integer(_) | Token::Float(_)) {
            return self.numeric_run();
        }
        match self.advance() {
            Token::Boolean(b) => Ok(Expr::Lit(Value::Atom(Atom::Boolean(*b)))),
            Token::Character(c) => Ok(Expr::Lit(Value::Atom(Atom::Character(*c)))),
            Token::Sym(s) => Ok(Expr::Lit(Value::Atom(Atom::Symbol(Sym::intern(s))))),
            Token::SymList(names) => Ok(vector(Column::Sym(
                names.iter().map(|s| Sym::intern(s)).collect(),
            ))),
            Token::Str(s) => Ok(vector(Column::Char(s.chars().collect()))),
            Token::BoolList(bits) => Ok(vector(Column::Bool(bits.clone()))),
            Token::Symbol(s) => Ok(Expr::Symbol(s.clone())),
            Token::LeftParen => {
                if self.check(&Token::RightParen) || self.check(&Token::Semicolon) {
                    return Err(QError::Nyi("general lists".into()));
                }
                let expr = self.expression()?;
                if self.check(&Token::Semicolon) {
                    return Err(QError::Nyi("general lists".into()));
                }
                if !self.match_tokens(&[Token::RightParen]) {
                    return Err(nyi_token(self.peek())
                        .unwrap_or_else(|| QError::parse("expected ')' after expression")));
                }
                Ok(expr)
            }
            token => {
                Err(nyi_token(token)
                    .unwrap_or_else(|| QError::parse(format!("unexpected {}", token))))
            }
        }
    }

    /// A run of juxtaposed numbers: one atom, or a long vector, or a float
    /// vector when any element is a float (long nulls become `0n`).
    fn numeric_run(&mut self) -> QResult<Expr> {
        let start = self.current;
        while matches!(self.peek(), Token::Integer(_) | Token::Float(_)) {
            self.current += 1;
        }
        let run = &self.tokens[start..self.current];
        let to_float = |t: &Token| match t {
            Token::Integer(i64::MIN) => f64::NAN,
            Token::Integer(n) => *n as f64,
            Token::Float(f) => *f,
            _ => unreachable!("run holds numbers only"),
        };
        let any_float = run.iter().any(|t| matches!(t, Token::Float(_)));
        Ok(match (run, any_float) {
            ([Token::Integer(n)], _) => Expr::Lit(Value::Atom(Atom::Integer(*n))),
            ([Token::Float(f)], _) => Expr::Lit(Value::Atom(Atom::Float(*f))),
            (run, true) => vector(Column::Float(run.iter().map(to_float).collect())),
            (run, false) => vector(Column::Long(
                run.iter()
                    .map(|t| match t {
                        Token::Integer(n) => *n,
                        _ => unreachable!("no floats in this run"),
                    })
                    .collect(),
            )),
        })
    }

    fn match_tokens(&mut self, types: &[Token]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(self.peek()) == std::mem::discriminant(token_type)
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}

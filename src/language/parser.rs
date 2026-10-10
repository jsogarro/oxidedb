use crate::error::{QError, QResult};
use crate::language::{
    ast::{Expr, UnaryOperator, Verb},
    lexer::Token,
};
use crate::types::atom::Atom;

fn adverb_nyi(token: &Token) -> QError {
    QError::Nyi(format!("adverb '{}'", token))
}

/// Lexed tokens (list literals, verbs, punctuation) the parser cannot handle yet.
fn literal_nyi(token: &Token) -> Option<QError> {
    let detail = match token {
        Token::Str(_) => "strings (a character literal holds exactly one character)",
        Token::Sym(_) | Token::SymList(_) => "symbols",
        Token::BoolList(_) => "boolean lists",
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
        | Token::EachPrior
        | Token::LeftBrace
        | Token::RightBrace
        | Token::Quote => return Some(QError::Nyi(token.to_string())),
        _ => return None,
    };
    Some(QError::Nyi(detail.into()))
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
            if matches!(self.peek(), Token::Over | Token::Scan) {
                return Err(adverb_nyi(self.peek()));
            }
            if let Some(err) = literal_nyi(self.peek()) {
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
            operands.push(operand);
            if !self.match_tokens(&[Token::Plus, Token::Minus, Token::Multiply, Token::Divide]) {
                break;
            }
            operators.push(match self.previous() {
                Token::Plus => Verb::Add,
                Token::Minus => Verb::Subtract,
                Token::Multiply => Verb::Multiply,
                Token::Divide => Verb::Divide,
                _ => unreachable!(),
            });
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
            // Monadic minus takes its whole right side, as in q: -x+3 is -(x+3).
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
        match self.advance() {
            Token::Integer(n) => Ok(Expr::Atom(Atom::Integer(*n))),
            Token::Float(f) => Ok(Expr::Atom(Atom::Float(*f))),
            Token::Boolean(b) => Ok(Expr::Atom(Atom::Boolean(*b))),
            Token::Character(c) => Ok(Expr::Atom(Atom::Character(*c))),
            Token::Symbol(s) => Ok(Expr::Symbol(s.clone())),
            Token::LeftParen => {
                let expr = self.expression()?;
                if !self.match_tokens(&[Token::RightParen]) {
                    return Err(QError::parse("expected ')' after expression"));
                }
                Ok(expr)
            }
            token @ (Token::Over | Token::Scan) => Err(adverb_nyi(token)),
            token => Err(literal_nyi(token)
                .unwrap_or_else(|| QError::parse(format!("unexpected {}", token)))),
        }
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

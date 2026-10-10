use crate::language::{
    ast::{BinaryOperator, Expr, UnaryOperator},
    lexer::Token,
};
use crate::types::atom::Atom;
use anyhow::{anyhow, Result};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Expr> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
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

    fn binary_expression(&mut self) -> Result<Expr> {
        let left = self.unary()?;

        if self.match_tokens(&[Token::Plus, Token::Minus, Token::Multiply, Token::Divide]) {
            let operator = match self.previous() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                Token::Multiply => BinaryOperator::Multiply,
                Token::Divide => BinaryOperator::Divide,
                _ => unreachable!(),
            };
            // Right-associative: recursively parse the right side
            let right = self.expression()?;
            Ok(Expr::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_tokens(&[Token::Minus]) {
            let expr = self.unary()?;
            return Ok(Expr::UnaryOp {
                operator: UnaryOperator::Negate,
                operand: Box::new(expr),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        match self.advance() {
            Token::Integer(n) => Ok(Expr::Atom(Atom::Integer(*n))),
            Token::Float(f) => Ok(Expr::Atom(Atom::Float(*f))),
            Token::Boolean(b) => Ok(Expr::Atom(Atom::Boolean(*b))),
            Token::Character(c) => Ok(Expr::Atom(Atom::Character(*c))),
            Token::Symbol(s) => Ok(Expr::Symbol(s.clone())),
            Token::LeftParen => {
                let expr = self.expression()?;
                if !self.match_tokens(&[Token::RightParen]) {
                    return Err(anyhow!("Expected ')' after expression"));
                }
                Ok(expr)
            }
            token => Err(anyhow!("Unexpected token: {:?}", token)),
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

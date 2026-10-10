use crate::language::{
    ast::{BinaryOperator, Expr, UnaryOperator},
    lexer::Token,
};
use crate::types::atom::Atom;
use anyhow::{anyhow, Result};

fn adverb_nyi(token: &Token) -> anyhow::Error {
    anyhow!("adverb '{}' not yet implemented", token)
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

    pub fn parse(&mut self) -> Result<Expr> {
        let expr = self.expression()?;
        if !self.is_at_end() {
            if matches!(self.peek(), Token::Over | Token::Scan) {
                return Err(adverb_nyi(self.peek()));
            }
            return Err(anyhow!(
                "Unexpected token after expression: {:?}",
                self.peek()
            ));
        }
        Ok(expr)
    }

    fn expression(&mut self) -> Result<Expr> {
        if self.depth >= MAX_DEPTH {
            return Err(anyhow!("expression nested too deeply"));
        }
        self.depth += 1;
        let result = self.assignment();
        self.depth -= 1;
        result
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
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                Token::Multiply => BinaryOperator::Multiply,
                Token::Divide => BinaryOperator::Divide,
                _ => unreachable!(),
            });
            // The AST is still a right-nested tree, which evaluate and drop recurse over.
            if operators.len() >= MAX_CHAIN {
                return Err(anyhow!("expression too long"));
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

    fn unary(&mut self) -> Result<Expr> {
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

    fn primary(&mut self) -> Result<Expr> {
        if self.is_at_end() {
            return Err(anyhow!("Unexpected end of input"));
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
                    return Err(anyhow!("Expected ')' after expression"));
                }
                Ok(expr)
            }
            token @ (Token::Over | Token::Scan) => Err(adverb_nyi(token)),
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

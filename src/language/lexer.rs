use anyhow::{anyhow, Result};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Character(char),
    Symbol(String),

    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,

    // Adverbs (lexed; not yet parsed)
    Over,
    Scan,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Semicolon,
    Colon,

    // Assignment
    Assignment,

    // End of file
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Integer(n) => write!(f, "{}", n),
            Token::Float(n) => write!(f, "{}", n),
            Token::Boolean(b) => write!(f, "{}", if *b { "1b" } else { "0b" }),
            Token::Character(c) => write!(f, "\"{}\"", c),
            Token::Symbol(s) => write!(f, "{}", s),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "%"),
            Token::Over => write!(f, "/"),
            Token::Scan => write!(f, "\\"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::Assignment => write!(f, ":"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.first().copied();

        Self {
            input: chars,
            position: 0,
            current_char,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            match self.next_token(tokens.last())? {
                Token::Eof => {
                    tokens.push(Token::Eof);
                    break;
                }
                token => tokens.push(token),
            }
        }

        Ok(tokens)
    }

    fn next_token(&mut self, prev: Option<&Token>) -> Result<Token> {
        self.skip_whitespace();

        match self.current_char {
            None => Ok(Token::Eof),
            Some(ch) => match ch {
                '+' => {
                    self.advance();
                    Ok(Token::Plus)
                }
                '-' if self.starts_negative_literal(prev) => self.read_number(),
                '-' => {
                    self.advance();
                    Ok(Token::Minus)
                }
                '*' => {
                    self.advance();
                    Ok(Token::Multiply)
                }
                '%' => {
                    self.advance();
                    Ok(Token::Divide)
                }
                // A comment `/` was already consumed by skip_whitespace, so any
                // `/` reaching here is glued to the previous token.
                '/' => {
                    self.advance();
                    Ok(Token::Over)
                }
                '\\' => {
                    self.advance();
                    Ok(Token::Scan)
                }
                '.' if self.next_is_digit() => self.read_number(),
                '(' => {
                    self.advance();
                    Ok(Token::LeftParen)
                }
                ')' => {
                    self.advance();
                    Ok(Token::RightParen)
                }
                '[' => {
                    self.advance();
                    Ok(Token::LeftBracket)
                }
                ']' => {
                    self.advance();
                    Ok(Token::RightBracket)
                }
                ';' => {
                    self.advance();
                    Ok(Token::Semicolon)
                }
                ':' => {
                    self.advance();
                    Ok(Token::Colon)
                }
                '"' => self.read_character(),
                ch if ch.is_ascii_digit() => self.read_number(),
                ch if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
                _ => Err(anyhow!("Unexpected character: {}", ch)),
            },
        }
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    /// Skips whitespace and `/` comments. q rule: `/` at line start or after
    /// whitespace comments out the rest of the line; a glued `/` is the over adverb.
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '/'
                && (self.position == 0 || self.input[self.position - 1].is_whitespace())
            {
                while self.current_char.is_some_and(|c| c != '\n') {
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn next_is_digit(&self) -> bool {
        self.input
            .get(self.position + 1)
            .is_some_and(char::is_ascii_digit)
    }

    /// q rule: `-` glued to a digit is part of the number at input start or
    /// after a non-noun; after a noun only when preceded by whitespace (`2 -1`).
    fn starts_negative_literal(&self, prev: Option<&Token>) -> bool {
        let next = self.input.get(self.position + 1);
        let dot_digit = next == Some(&'.')
            && self
                .input
                .get(self.position + 2)
                .is_some_and(char::is_ascii_digit);
        if !(next.is_some_and(char::is_ascii_digit) || dot_digit) {
            return false;
        }
        let spaced = self.position > 0 && self.input[self.position - 1].is_whitespace();
        match prev {
            None => true,
            Some(
                Token::Integer(_)
                | Token::Float(_)
                | Token::Boolean(_)
                | Token::Character(_)
                | Token::Symbol(_)
                | Token::RightParen
                | Token::RightBracket,
            ) => spaced,
            Some(_) => true,
        }
    }

    fn read_number(&mut self) -> Result<Token> {
        let mut number = String::new();
        let mut is_float = false;

        if self.current_char == Some('-') {
            number.push('-');
            self.advance();
        }

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                is_float = true;
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if (number == "0" || number == "1")
            && self.current_char == Some('b')
            && !self
                .input
                .get(self.position + 1)
                .is_some_and(|c| c.is_alphanumeric() || *c == '_')
        {
            self.advance();
            return Ok(Token::Boolean(number == "1"));
        }

        if is_float {
            let value = number
                .parse::<f64>()
                .map_err(|_| anyhow!("Invalid float: {}", number))?;
            Ok(Token::Float(value))
        } else {
            let value = number
                .parse::<i64>()
                .map_err(|_| anyhow!("Invalid integer: {}", number))?;
            Ok(Token::Integer(value))
        }
    }

    fn read_identifier(&mut self) -> Result<Token> {
        let mut identifier = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Ok(Token::Symbol(identifier))
    }

    fn read_character(&mut self) -> Result<Token> {
        self.advance(); // Skip opening quote

        let ch = self
            .current_char
            .ok_or_else(|| anyhow!("Unterminated character literal"))?;
        self.advance();

        if self.current_char == Some('"') {
            self.advance(); // Skip closing quote
            Ok(Token::Character(ch))
        } else {
            Err(anyhow!("Unterminated character literal"))
        }
    }
}

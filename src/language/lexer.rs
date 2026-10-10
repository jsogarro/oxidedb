use crate::error::{QError, QResult};
use crate::types::atom::Atom;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Character(char),
    Symbol(String),
    /// `` `abc ``; a lone backtick is the null symbol `Sym("")`.
    Sym(String),
    /// A glued run of two or more symbols: `` `a`b`c ``.
    SymList(Vec<String>),
    /// `"abc"` or `""` (a one-character literal is `Character`).
    Str(String),
    /// `101b`: two or more digits of 0/1.
    BoolList(Vec<bool>),

    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Equal,
    Less,
    Greater,
    NotEqual,
    LessEqual,
    GreaterEqual,
    Hash,
    Comma,
    Bang,
    Dollar,
    At,

    // Adverbs (lexed; not yet parsed)
    Over,
    Scan,
    EachPrior,
    EachRight,
    EachLeft,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Quote,
    Semicolon,
    Colon,
    DoubleColon,

    // Assignment
    Assignment,

    // End of file
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Integer(n) => write!(f, "{}", Atom::Integer(*n)),
            Token::Float(n) => write!(f, "{}", Atom::Float(*n)),
            Token::Boolean(b) => write!(f, "{}", Atom::Boolean(*b)),
            Token::Character(c) => write!(f, "{}", Atom::Character(*c)),
            Token::Symbol(s) => write!(f, "{}", s),
            Token::Sym(s) => write!(f, "`{s}"),
            Token::SymList(names) => names.iter().try_for_each(|s| write!(f, "`{s}")),
            Token::Str(s) => {
                write!(f, "\"")?;
                for c in s.chars() {
                    match c {
                        '"' => write!(f, "\\\"")?,
                        '\\' => write!(f, "\\\\")?,
                        '\n' => write!(f, "\\n")?,
                        '\t' => write!(f, "\\t")?,
                        '\r' => write!(f, "\\r")?,
                        c => write!(f, "{c}")?,
                    }
                }
                write!(f, "\"")
            }
            Token::BoolList(bits) => {
                bits.iter()
                    .try_for_each(|b| write!(f, "{}", u8::from(*b)))?;
                write!(f, "b")
            }
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "%"),
            Token::Equal => write!(f, "="),
            Token::Less => write!(f, "<"),
            Token::Greater => write!(f, ">"),
            Token::NotEqual => write!(f, "<>"),
            Token::LessEqual => write!(f, "<="),
            Token::GreaterEqual => write!(f, ">="),
            Token::Hash => write!(f, "#"),
            Token::Comma => write!(f, ","),
            Token::Bang => write!(f, "!"),
            Token::Dollar => write!(f, "$"),
            Token::At => write!(f, "@"),
            Token::EachPrior => write!(f, "':"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::Quote => write!(f, "'"),
            Token::EachRight => write!(f, "/:"),
            Token::EachLeft => write!(f, "\\:"),
            Token::DoubleColon => write!(f, "::"),
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

fn bad_identifier_char(ch: char) -> QError {
    if ch == '_' {
        QError::parse("invalid identifier: a name must start with a letter, not '_'")
    } else {
        QError::parse(format!("invalid identifier: non-ASCII character '{ch}'; names are ASCII letters, digits and '_'"))
    }
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    /// Source span of the last literal token, to catch a literal glued to the next.
    last_literal: Option<(usize, usize)>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.first().copied();

        Self {
            input: chars,
            position: 0,
            current_char,
            last_literal: None,
        }
    }

    pub fn tokenize(&mut self) -> QResult<Vec<Token>> {
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

    /// Like `lex_token`, but a literal must not run straight into another one
    /// (`1.5.5`, `"ab""cd"`, `1"ab"`): later slices join juxtaposed literals
    /// into vectors, so glued ones are rejected here.
    fn next_token(&mut self, prev: Option<&Token>) -> QResult<Token> {
        self.skip_whitespace();
        let start = self.position;
        let glued_from = self
            .last_literal
            .filter(|&(_, end)| end == start && self.starts_literal())
            .map(|(from, _)| from);
        let token = self.lex_token(prev)?;
        if let Some(from) = glued_from {
            let text: String = self.input[from..start].iter().collect();
            return Err(QError::parse(format!("invalid literal: {text}...")));
        }
        self.last_literal = matches!(
            token,
            Token::Integer(_)
                | Token::Float(_)
                | Token::Boolean(_)
                | Token::Character(_)
                | Token::Sym(_)
                | Token::SymList(_)
                | Token::Str(_)
                | Token::BoolList(_)
        )
        .then_some((start, self.position));
        Ok(token)
    }

    fn starts_literal(&self) -> bool {
        match self.current_char {
            Some(c) => {
                c.is_ascii_digit() || matches!(c, '"' | '`') || (c == '.' && self.next_is_digit())
            }
            None => false,
        }
    }

    fn lex_token(&mut self, prev: Option<&Token>) -> QResult<Token> {
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
                '/' => self.adverb(Token::Over, Token::EachRight),
                '\\' => self.adverb(Token::Scan, Token::EachLeft),
                '=' => self.single(Token::Equal),
                '<' if self.next_char() == Some('>') => self.pair(Token::NotEqual),
                '<' if self.next_char() == Some('=') => self.pair(Token::LessEqual),
                '<' => self.single(Token::Less),
                '>' if self.next_char() == Some('=') => self.pair(Token::GreaterEqual),
                '>' => self.single(Token::Greater),
                '#' => self.single(Token::Hash),
                ',' => self.single(Token::Comma),
                '!' => self.single(Token::Bang),
                '{' => self.single(Token::LeftBrace),
                '}' => self.single(Token::RightBrace),
                '$' => self.single(Token::Dollar),
                '@' => self.single(Token::At),
                '\'' => self.adverb(Token::Quote, Token::EachPrior),
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
                ':' => self.adverb(Token::Colon, Token::DoubleColon),
                '"' => self.read_character(),
                ch if ch.is_ascii_digit() => self.read_number(),
                ch if ch.is_ascii_alphabetic() => self.read_identifier(),
                ch if ch.is_alphanumeric() || ch == '_' => Err(bad_identifier_char(ch)),
                '`' => self.read_symbols(),
                _ => Err(QError::parse(format!("unexpected character: {ch}"))),
            },
        }
    }

    fn single(&mut self, token: Token) -> QResult<Token> {
        self.advance();
        Ok(token)
    }

    fn pair(&mut self, token: Token) -> QResult<Token> {
        self.advance();
        self.single(token)
    }

    /// `alone`, or `with_colon` when a `:` follows directly (`/` `/:`).
    fn adverb(&mut self, alone: Token, with_colon: Token) -> QResult<Token> {
        if self.next_char() == Some(':') {
            self.pair(with_colon)
        } else {
            self.single(alone)
        }
    }

    fn next_char(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
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

    fn ident_char_at(&self, offset: usize) -> bool {
        self.input
            .get(self.position + offset)
            .is_some_and(|c| is_ident_char(*c))
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
        if !(next.is_some_and(char::is_ascii_digit) || dot_digit) || self.bool_literal_at(1) {
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
                | Token::Sym(_)
                | Token::SymList(_)
                | Token::Str(_)
                | Token::BoolList(_)
                | Token::RightParen
                | Token::RightBracket
                | Token::RightBrace,
            ) => spaced,
            Some(_) => true,
        }
    }

    /// True when the digits starting `offset` ahead are a boolean literal
    /// (`1b`, `101b`), so a minus before them stays an operator.
    fn bool_literal_at(&self, offset: usize) -> bool {
        let rest = &self.input[self.position + offset..];
        let digits = rest.iter().take_while(|c| c.is_ascii_digit()).count();
        rest.get(digits) == Some(&'b') && !rest.get(digits + 1).is_some_and(|c| is_ident_char(*c))
    }

    /// A number must not run straight into a letter or `_` (`2x`, `1E3`, `1j`):
    /// like a glued literal, that is an invalid literal rather than two tokens.
    fn read_number(&mut self) -> QResult<Token> {
        let start = self.position;
        let token = self.read_number_body()?;
        if self.current_char.is_some_and(is_ident_char) {
            let text: String = self.input[start..=self.position].iter().collect();
            return Err(QError::parse(format!("invalid literal: {text}...")));
        }
        Ok(token)
    }

    fn read_number_body(&mut self) -> QResult<Token> {
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
            && !self.ident_char_at(1)
        {
            self.advance();
            return Ok(Token::Boolean(number == "1"));
        }

        // `101b`: two or more digits glued to a `b`.
        if self.current_char == Some('b')
            && number.len() >= 2
            && number.chars().all(|c| c.is_ascii_digit())
        {
            let bits: Option<Vec<bool>> = number
                .chars()
                .map(|c| match c {
                    '0' => Some(false),
                    '1' => Some(true),
                    _ => None,
                })
                .collect();
            return match bits {
                Some(bits) if !self.ident_char_at(1) => {
                    self.advance();
                    Ok(Token::BoolList(bits))
                }
                _ => Err(QError::parse(format!("invalid literal: {number}b..."))),
            };
        }

        // Null and infinity literals: `0N` `0n` `0w` (optionally negative).
        if number == "0" || number == "-0" {
            let neg = number.starts_with('-');
            let literal = match self.current_char {
                Some('N') => Some(Token::Integer(i64::MIN)),
                Some('n') => Some(Token::Float(f64::NAN)),
                Some('w') => Some(Token::Float(if neg {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                })),
                Some('W') => return Err(QError::Nyi("0W (long infinity)".into())),
                _ => None,
            };
            if let Some(token) = literal {
                if self.ident_char_at(1) {
                    return Err(QError::parse(format!(
                        "invalid literal: {}{}...",
                        number,
                        self.current_char.unwrap_or(' ')
                    )));
                }
                self.advance();
                return Ok(token);
            }
        }

        if self.current_char == Some('e') {
            is_float = true;
            number.push('e');
            self.advance();
            if let Some(sign @ ('+' | '-')) = self.current_char {
                number.push(sign);
                self.advance();
            }
            if !self.current_char.is_some_and(|c| c.is_ascii_digit()) {
                return Err(QError::parse(
                    "invalid float: exponent needs digits after 'e'",
                ));
            }
            while let Some(ch) = self.current_char.filter(char::is_ascii_digit) {
                number.push(ch);
                self.advance();
            }
        }

        if self.current_char == Some('f') && !self.ident_char_at(1) {
            is_float = true;
            self.advance();
        }

        // A `b` that did not make a boolean literal (`2b`, `1.0b`, `1e3b`).
        if self.current_char == Some('b') && !self.ident_char_at(1) {
            return Err(QError::parse(format!("invalid literal: {number}b...")));
        }

        if is_float {
            let value = number
                .parse::<f64>()
                .map_err(|_| QError::parse(format!("invalid float: {number}")))?;
            if !value.is_finite() {
                return Err(QError::parse(format!("float out of range: {number}")));
            }
            Ok(Token::Float(value))
        } else {
            let value = number
                .parse::<i64>()
                .map_err(|_| QError::parse(format!("invalid integer: {number}")))?;
            Ok(Token::Integer(value))
        }
    }

    /// Called only on an ASCII letter: consumes it unconditionally, so this can
    /// never return without advancing.
    fn read_identifier(&mut self) -> QResult<Token> {
        let mut identifier = String::new();
        while let Some(ch) = self.current_char {
            if identifier.is_empty() || is_ident_char(ch) {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        Ok(Token::Symbol(identifier))
    }

    /// `` `a ``, `` ` `` or a glued run `` `a`b ``; the lexer is on a backtick.
    fn read_symbols(&mut self) -> QResult<Token> {
        let mut names = Vec::new();
        while self.current_char == Some('`') {
            self.advance();
            let mut name = String::new();
            while let Some(ch) = self
                .current_char
                .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.'))
            {
                name.push(ch);
                self.advance();
            }
            names.push(name);
        }
        if let Some(ch) = self.current_char.filter(|c| c.is_alphanumeric()) {
            return Err(QError::parse(format!(
                "invalid symbol: non-ASCII character '{ch}'; symbol names are ASCII letters, digits, '_' and '.'"
            )));
        }
        Ok(if names.len() == 1 {
            Token::Sym(names.remove(0))
        } else {
            Token::SymList(names)
        })
    }

    /// `"c"` is a character, anything else between quotes a string. A `\` escape
    /// (`\" \\ \n \t \r`) is one character of the body.
    fn read_character(&mut self) -> QResult<Token> {
        self.advance(); // Skip opening quote

        if self.current_char == Some('"') {
            self.advance();
            return Ok(Token::Str(String::new()));
        }

        let unterminated = || QError::parse("unterminated character literal");
        let mut body = Vec::new();
        loop {
            let ch = self.current_char.ok_or_else(unterminated)?;
            self.advance();
            match ch {
                '"' => break,
                '\\' => {
                    let esc = self.current_char.ok_or_else(unterminated)?;
                    self.advance();
                    body.push(match esc {
                        '"' | '\\' => esc,
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        _ => {
                            return Err(QError::parse(format!("invalid escape: \\{esc} in string")))
                        }
                    });
                }
                _ => body.push(ch),
            }
        }
        Ok(match body[..] {
            [c] => Token::Character(c),
            _ => Token::Str(body.into_iter().collect()),
        })
    }
}

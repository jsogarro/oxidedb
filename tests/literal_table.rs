//! A numeric literal is a body (`-?digits[.digits][e[+-]digits]`) plus the whole
//! trailing identifier run, decided together: a valid (body, suffix) pair is a
//! token, anything else is one `invalid literal` error naming the body and the
//! first character of the run.

use oxidedb::language::lexer::Token;
use oxidedb::{Lexer, QError};

fn lex(src: &str) -> Result<Vec<Token>, QError> {
    Lexer::new(src).tokenize()
}

fn invalid(src: &str, shown: &str) {
    assert_eq!(
        lex(src),
        Err(QError::parse(format!("invalid literal: {shown}..."))),
        "{src}"
    );
}

#[test]
fn malformed_literals_from_the_issue_are_one_clear_error() {
    invalid("1E3", "1E");
    invalid("1F", "1F");
    invalid("2b", "2b");
    invalid("1ff", "1f");
    invalid("1e3e3", "1e3e");
    invalid("0x1", "0x");
    invalid("1_000", "1_");
    invalid("1foo", "1f");
}

#[test]
fn an_invalid_suffix_wins_over_a_range_error_on_the_body() {
    invalid("99999999999999999999x", "99999999999999999999x");
    invalid("99999999999999999999b", "99999999999999999999b");
    invalid("1e999x", "1e999x");
    invalid("1e999b", "1e999b");
}

#[test]
fn a_valid_suffix_still_reports_the_range_error() {
    assert_eq!(
        lex("1e999f"),
        Err(QError::parse("float out of range: 1e999"))
    );
    assert_eq!(
        lex("99999999999999999999"),
        Err(QError::parse("invalid integer: 99999999999999999999"))
    );
}

#[test]
fn valid_pairs_are_unchanged() {
    let one = |src: &str| {
        let mut t = lex(src).unwrap();
        assert_eq!(t.pop(), Some(Token::Eof));
        t
    };
    assert_eq!(one("1b"), vec![Token::Boolean(true)]);
    assert_eq!(one("101b"), vec![Token::BoolList(vec![true, false, true])]);
    assert_eq!(one("1f"), vec![Token::Float(1.0)]);
    assert_eq!(one("-1.5e2f"), vec![Token::Float(-150.0)]);
    assert_eq!(one("1e3"), vec![Token::Float(1000.0)]);
    assert_eq!(one("0N"), vec![Token::Integer(i64::MIN)]);
    assert_eq!(one("-0w"), vec![Token::Float(f64::NEG_INFINITY)]);
    assert!(matches!(&one("0n")[..], [Token::Float(f)] if f.is_nan()));
}

#[test]
fn bad_body_pairs_name_body_and_first_suffix_char() {
    invalid("1.5b", "1.5b");
    invalid("102b", "102b");
    invalid("0Nx", "0N");
    invalid("0.5N", "0.5N");
    invalid("1j", "1j");
}

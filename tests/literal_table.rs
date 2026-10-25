//! A numeric literal is a body (`-?digits*[.digits*][e[+-]digits]`) plus the whole
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

#[test]
fn only_a_zero_body_takes_the_null_and_infinity_suffixes() {
    invalid("1W", "1W");
    invalid("1.5W", "1.5W");
    invalid("1n", "1n");
    invalid("1w", "1w");
    invalid("00N", "00N");
}

#[test]
fn zero_w_is_nyi_only_when_nothing_is_glued_to_it() {
    let nyi = Err(QError::Nyi("0W (long infinity)".into()));
    assert_eq!(lex("0W"), nyi);
    assert_eq!(lex("-0W"), nyi);
    invalid("0Wx", "0W");
    invalid("0W0", "0W");
}

#[test]
fn bodies_with_a_sign_or_fraction_still_reject_a_bad_suffix() {
    invalid("-1x", "-1x");
    invalid("-1.5b", "-1.5b");
    invalid(".5b", ".5b");
    invalid("1.b", "1.b");
    invalid("1e3_", "1e3_");
    invalid("1f 2", "1f");
    assert_eq!(
        lex("1ex"),
        Err(QError::parse(
            "invalid float: exponent needs digits after 'e'"
        ))
    );
}

#[test]
fn a_valid_literal_stops_at_the_first_non_identifier_character() {
    assert_eq!(
        lex("1f-2").unwrap(),
        vec![
            Token::Float(1.0),
            Token::Minus,
            Token::Integer(2),
            Token::Eof
        ]
    );
    assert_eq!(
        lex("110b").unwrap(),
        vec![Token::BoolList(vec![true, true, false]), Token::Eof]
    );
    assert_eq!(lex("0b").unwrap(), vec![Token::Boolean(false), Token::Eof]);
    assert_eq!(
        lex("00b").unwrap(),
        vec![Token::BoolList(vec![false, false]), Token::Eof]
    );
}

#[test]
fn every_body_shape_lexes_to_its_value() {
    let one = |src: &str| {
        let mut t = lex(src).unwrap();
        assert_eq!(t.pop(), Some(Token::Eof));
        t
    };
    assert_eq!(one("0w"), vec![Token::Float(f64::INFINITY)]);
    assert_eq!(one("1e+3"), vec![Token::Float(1000.0)]);
    assert_eq!(one("1e-3"), vec![Token::Float(0.001)]);
    assert_eq!(one("1.5"), vec![Token::Float(1.5)]);
    assert_eq!(one("1."), vec![Token::Float(1.0)]);
    assert_eq!(one("1.e3"), vec![Token::Float(1000.0)]);
    assert_eq!(one(".5"), vec![Token::Float(0.5)]);
    assert_eq!(one("-.5"), vec![Token::Float(-0.5)]);
}

#[test]
fn a_long_glued_run_is_still_one_error() {
    invalid(&format!("1{}", "x".repeat(100_000)), "1x");
}

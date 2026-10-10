use oxidedb::language::lexer::Token;
use oxidedb::{Interpreter, Lexer, Parser, QError};

fn run(src: &str) -> Result<Option<oxidedb::Value>, QError> {
    Interpreter::new().eval_line(src)
}

fn err(src: &str) -> QError {
    run(src).expect_err(src)
}

#[test]
fn errors_display_q_style() {
    let cases = [
        (QError::Type, "'type"),
        (QError::Length, "'length"),
        (QError::Rank, "'rank"),
        (QError::Index, "'index"),
        (QError::Domain, "'domain"),
        (QError::Nyi("adverb '/'".into()), "'nyi: adverb '/'"),
        (QError::Parse("bad".into()), "'parse: bad"),
        (QError::Undefined("x".into()), "'x (Undefined variable)"),
        (QError::Stack, "'stack"),
        (QError::Overflow, "'overflow"),
        (QError::Signal("boom".into()), "'boom"),
    ];
    for (e, text) in cases {
        assert_eq!(e.to_string(), text);
    }
}

#[test]
fn errors_parse_kind() {
    // lexer failures
    for src in ["1e", "1e999", "-1e999", "_a:1", "é:1", "\"a", "^", "1x0N"] {
        assert!(matches!(err(src), QError::Parse(_)), "{src}");
    }
    assert_eq!(
        err("1e999"),
        QError::Parse("float out of range: 1e999".into())
    );
    // parser failures
    for src in ["1 +", "(1", "+1", ")", "1 2", "1 )"] {
        assert!(matches!(err(src), QError::Parse(_)), "{src}");
    }
    assert_eq!(
        err("(1").to_string(),
        "'parse: expected ')' after expression"
    );
    assert_eq!(err("1 +").to_string(), "'parse: unexpected end of input");
    assert_eq!(err("+1").to_string(), "'parse: unexpected +");
    // limits keep their message text inside Parse
    let deep = format!("{}1{}", "(".repeat(200), ")".repeat(200));
    assert_eq!(
        err(&deep),
        QError::Parse("expression nested too deeply".into())
    );
    let long = format!("{}1", "1+".repeat(2000));
    assert_eq!(err(&long), QError::Parse("expression too long".into()));
}

#[test]
fn errors_type_kind() {
    for src in [
        "1+\"a\"",
        "1b+1",
        "\"a\"+\"b\"",
        "-(1b)",
        "-(\"a\")",
        "1.5*1b",
    ] {
        assert_eq!(err(src), QError::Type, "{src}");
    }
    assert_eq!(err("1b+1").to_string(), "'type");
}

#[test]
fn errors_overflow_kind() {
    for src in [
        "9223372036854775807+1",
        "-9223372036854775807-2",
        "4611686018427387904*2",
        "-9223372036854775807-1",
    ] {
        assert_eq!(err(src), QError::Overflow, "{src}");
    }
}

#[test]
fn errors_undefined_kind() {
    assert_eq!(err("y"), QError::Undefined("y".into()));
    assert_eq!(err("1+zed"), QError::Undefined("zed".into()));
    assert_eq!(err("y").to_string(), "'y (Undefined variable)");
}

#[test]
fn errors_nyi_kind() {
    assert_eq!(err("6/2"), QError::Nyi("adverb '/'".into()));
    assert_eq!(err("6\\2"), QError::Nyi("adverb '\\'".into()));
    assert_eq!(err("6/2").to_string(), "'nyi: adverb '/'");
    for src in ["0W", "-0W"] {
        assert_eq!(err(src), QError::Nyi("0W (long infinity)".into()), "{src}");
    }
}

#[test]
fn errors_token_source_form() {
    let cases = [
        (Token::Integer(-1), "-1"),
        (Token::Integer(42), "42"),
        (Token::Integer(i64::MIN), "0N"),
        (Token::Float(845.0), "845f"),
        (Token::Float(0.5), "0.5"),
        (Token::Float(f64::NAN), "0n"),
        (Token::Float(f64::INFINITY), "0w"),
        (Token::Float(f64::NEG_INFINITY), "-0w"),
        (Token::Boolean(true), "1b"),
        (Token::Boolean(false), "0b"),
        (Token::Character('a'), "\"a\""),
        (Token::Character('"'), "\"\\\"\""),
        (Token::Symbol("abc".into()), "abc"),
        (Token::Plus, "+"),
        (Token::Minus, "-"),
        (Token::Multiply, "*"),
        (Token::Divide, "%"),
        (Token::Over, "/"),
        (Token::Scan, "\\"),
        (Token::LeftParen, "("),
        (Token::RightParen, ")"),
        (Token::LeftBracket, "["),
        (Token::RightBracket, "]"),
        (Token::Semicolon, ";"),
        (Token::Colon, ":"),
        (Token::Assignment, ":"),
    ];
    for (t, text) in cases {
        assert_eq!(t.to_string(), text, "{t:?}");
    }
    // as they appear in messages
    let msgs = [
        ("2 -1", "'parse: unexpected -1 after expression"),
        ("0N 0N", "'parse: unexpected 0N after expression"),
        ("1 2.5", "'parse: unexpected 2.5 after expression"),
        ("1 1b", "'parse: unexpected 1b after expression"),
        ("1 x", "'parse: unexpected x after expression"),
        ("1 \"a\"", "'parse: unexpected \"a\" after expression"),
        ("1 0w", "'parse: unexpected 0w after expression"),
        ("1 )", "'parse: unexpected ) after expression"),
        ("1 ]", "'parse: unexpected ] after expression"),
    ];
    for (src, msg) in msgs {
        assert_eq!(err(src).to_string(), msg, "{src}");
    }
    // library entry points return QError directly
    let tokens: Result<Vec<Token>, QError> = Lexer::new("1 +").tokenize();
    let parsed: Result<_, QError> = Parser::new(tokens.unwrap()).parse();
    assert!(parsed.is_err());
}

//! Symbol, string and boolean-vector literal tokens (lexer only; the parser
//! still reports them as not yet implemented).
use oxidedb::language::lexer::Token;
use oxidedb::{Atom, Interpreter, Lexer, QError};

fn lex(src: &str) -> Vec<Token> {
    let mut tokens = Lexer::new(src).tokenize().unwrap();
    assert_eq!(tokens.pop(), Some(Token::Eof));
    tokens
}

fn lex_err(src: &str) -> QError {
    Lexer::new(src).tokenize().expect_err(src)
}

fn eval_err(src: &str) -> QError {
    Interpreter::new().eval_line(src).expect_err(src)
}

fn sym(s: &str) -> Token {
    Token::Sym(s.into())
}

fn syms(names: &[&str]) -> Token {
    Token::SymList(names.iter().map(|s| s.to_string()).collect())
}

fn parse(detail: &str) -> QError {
    QError::Parse(detail.into())
}

#[test]
fn lex_lists_symbol() {
    assert_eq!(lex("`abc"), vec![sym("abc")]);
    assert_eq!(lex("`"), vec![sym("")]);
    assert_eq!(lex("`a_1.b2"), vec![sym("a_1.b2")]);
    assert_eq!(lex("`1a"), vec![sym("1a")]);
    // the name stops at the first character outside [A-Za-z0-9_.]
    assert_eq!(lex("`a+1"), vec![sym("a"), Token::Plus, Token::Integer(1)]);
    assert_eq!(lex("`a)"), vec![sym("a"), Token::RightParen]);
    assert_eq!(lex("`a;"), vec![sym("a"), Token::Semicolon]);
}

#[test]
fn lex_lists_symbol_run() {
    assert_eq!(lex("`a`b`c"), vec![syms(&["a", "b", "c"])]);
    assert_eq!(lex("`a`b"), vec![syms(&["a", "b"])]);
    assert_eq!(lex("`a `b"), vec![sym("a"), sym("b")]);
    assert_eq!(lex("`a`"), vec![syms(&["a", ""])]);
    assert_eq!(lex("``"), vec![syms(&["", ""])]);
    assert_eq!(lex("`a`b `c`d"), vec![syms(&["a", "b"]), syms(&["c", "d"])]);
}

#[test]
fn lex_lists_string() {
    assert_eq!(lex("\"abc\""), vec![Token::Str("abc".into())]);
    assert_eq!(lex("\"\""), vec![Token::Str(String::new())]);
    assert_eq!(lex("\"a b\""), vec![Token::Str("a b".into())]);
    assert_eq!(lex("\"a\""), vec![Token::Character('a')]);
    assert_eq!(lex("\"é\""), vec![Token::Character('é')]);
    // no special case for `"""`: an empty string, then an unterminated one
    assert_eq!(lex_err("\"\"\""), parse("unterminated character literal"));
}

#[test]
fn lex_lists_string_escapes() {
    assert_eq!(
        lex(r#""a\"b\\c\nd\te\rf""#),
        vec![Token::Str("a\"b\\c\nd\te\rf".into())]
    );
    // an escaped single character is still a character
    assert_eq!(lex(r#""\n""#), vec![Token::Character('\n')]);
    assert_eq!(lex(r#""\t""#), vec![Token::Character('\t')]);
    assert_eq!(lex(r#""\r""#), vec![Token::Character('\r')]);
    assert_eq!(lex(r#""\\""#), vec![Token::Character('\\')]);
    assert_eq!(lex(r#""\"""#), vec![Token::Character('"')]);
    // an escaped quote does not close the string
    assert_eq!(lex(r#""ab\"""#), vec![Token::Str("ab\"".into())]);
    assert_eq!(lex_err(r#""a\x""#), parse("invalid escape: \\x in string"));
}

#[test]
fn lex_lists_string_unterminated() {
    for src in ["\"", "\"a", "\"abc", "\"ab\\\"", "\"ab\\", "x:\"ab"] {
        assert_eq!(
            lex_err(src),
            parse("unterminated character literal"),
            "{src}"
        );
    }
}

#[test]
fn lex_lists_bool_vector() {
    assert_eq!(lex("101b"), vec![Token::BoolList(vec![true, false, true])]);
    assert_eq!(lex("00b"), vec![Token::BoolList(vec![false, false])]);
    assert_eq!(lex("11b"), vec![Token::BoolList(vec![true, true])]);
    assert_eq!(lex("1b"), vec![Token::Boolean(true)]);
    assert_eq!(lex("0b"), vec![Token::Boolean(false)]);
    assert_eq!(
        lex("101b+1"),
        vec![
            Token::BoolList(vec![true, false, true]),
            Token::Plus,
            Token::Integer(1)
        ]
    );
    // a trailing identifier character means the literal is malformed
    assert_eq!(
        lex("1bb"),
        vec![Token::Integer(1), Token::Symbol("bb".into())]
    );
    for src in ["102b", "10b1", "10bx", "12b", "10b_"] {
        match lex_err(src) {
            QError::Parse(d) => assert!(d.starts_with("invalid literal"), "{src}: {d}"),
            e => panic!("{src}: {e:?}"),
        }
    }
    assert_eq!(lex_err("102b"), parse("invalid literal: 102b..."));
}

#[test]
fn lex_lists_interplay() {
    assert_eq!(lex("`a+1"), vec![sym("a"), Token::Plus, Token::Integer(1)]);
    assert_eq!(lex("x`a"), vec![Token::Symbol("x".into()), sym("a")]);
    // comments: spaced slash comments, glued slash is Over, slash in a string is text
    assert_eq!(lex("`a / c"), vec![sym("a")]);
    assert_eq!(lex("`a/"), vec![sym("a"), Token::Over]);
    assert_eq!(lex("\"a / b\""), vec![Token::Str("a / b".into())]);
    assert_eq!(lex("\"ab\" / c"), vec![Token::Str("ab".into())]);
    // negative literal after a noun only when spaced
    assert_eq!(lex("`a -1"), vec![sym("a"), Token::Integer(-1)]);
    assert_eq!(lex("`a`b -1"), vec![syms(&["a", "b"]), Token::Integer(-1)]);
    assert_eq!(
        lex("\"ab\" -1"),
        vec![Token::Str("ab".into()), Token::Integer(-1)]
    );
    assert_eq!(
        lex("101b -1"),
        vec![Token::BoolList(vec![true, false, true]), Token::Integer(-1)]
    );
    assert_eq!(lex("`a-1"), vec![sym("a"), Token::Minus, Token::Integer(1)]);
    assert_eq!(
        lex("`a`b-1"),
        vec![syms(&["a", "b"]), Token::Minus, Token::Integer(1)]
    );
    assert_eq!(
        lex("\"ab\"-1"),
        vec![Token::Str("ab".into()), Token::Minus, Token::Integer(1)]
    );
    assert_eq!(
        lex("101b-1"),
        vec![
            Token::BoolList(vec![true, false, true]),
            Token::Minus,
            Token::Integer(1)
        ]
    );
}

#[test]
fn lex_lists_parser_rejects_as_nyi() {
    let strings = "strings (a character literal holds exactly one character)";
    for src in ["\"ab\"", "\"\"", "x:\"Alice\"", "1 \"ab\"", "\"ab\" 1"] {
        assert_eq!(eval_err(src), QError::Nyi(strings.into()), "{src}");
    }
    for src in ["`a", "`", "`a`b`c", "x:`a", "1 `a", "(`a)", "1+`a"] {
        assert_eq!(eval_err(src), QError::Nyi("symbols".into()), "{src}");
    }
    for src in ["101b", "00b", "1+101b", "101b 1"] {
        assert_eq!(eval_err(src), QError::Nyi("boolean lists".into()), "{src}");
    }
    assert_eq!(eval_err("`a").to_string(), "'nyi: symbols");
}

#[test]
fn lex_lists_display_source_form() {
    assert_eq!(sym("a").to_string(), "`a");
    assert_eq!(sym("").to_string(), "`");
    assert_eq!(syms(&["a", "b"]).to_string(), "`a`b");
    assert_eq!(syms(&["a", ""]).to_string(), "`a`");
    assert_eq!(Token::Str("abc".into()).to_string(), "\"abc\"");
    assert_eq!(Token::Str(String::new()).to_string(), "\"\"");
    assert_eq!(
        Token::Str("a\"b\\c\nd\te\rf".into()).to_string(),
        r#""a\"b\\c\nd\te\rf""#
    );
    assert_eq!(Token::BoolList(vec![true, false, true]).to_string(), "101b");
}

#[test]
fn lex_lists_escapes_evaluate_as_characters() {
    for (src, c) in [
        (r#""\n""#, '\n'),
        (r#""\t""#, '\t'),
        (r#""\r""#, '\r'),
        (r#""\\""#, '\\'),
        (r#""\"""#, '"'),
    ] {
        let value = Interpreter::new().eval_line(src).unwrap().unwrap();
        assert_eq!(value, Atom::Character(c), "{src}");
    }
    // quote, backslash, quote: the backslash escapes the closing quote
    assert_eq!(
        Interpreter::new().eval_line(r#""\""#).unwrap_err(),
        parse("unterminated character literal")
    );
}

#[test]
fn lex_lists_literals_end_at_a_boundary() {
    // (source, text of the literal that ends too early)
    for (src, lit) in [
        ("1.5.5", "1.5"),
        ("1..5", "1."),
        (".5.5", ".5"),
        ("1e3.5", "1e3"),
        ("\"ab\"\"cd\"", "\"ab\""),
        ("\"ab\"1", "\"ab\""),
        ("1\"ab\"", "1"),
        ("10b.5", "10b"),
        ("`a\"b\"", "`a"),
        ("\"a\"`b", "\"a\""),
        ("\"a\"\"b\"", "\"a\""),
        ("1`a", "1"),
        ("1b`a", "1b"),
        ("101b\"a\"", "101b"),
        ("`a`b\"c\"", "`a`b"),
    ] {
        assert_eq!(
            lex_err(src),
            parse(&format!("invalid literal: {lit}...")),
            "{src}"
        );
    }
    assert_eq!(lex_err("101b1"), parse("invalid literal: 101b..."));
    // an operator, bracket or space after a literal is fine, as is an
    // identifier before a symbol
    assert_eq!(lex("1.5 .5"), vec![Token::Float(1.5), Token::Float(0.5)]);
    assert_eq!(
        lex("\"ab\" \"cd\""),
        vec![Token::Str("ab".into()), Token::Str("cd".into())]
    );
    assert_eq!(lex("`a)"), vec![sym("a"), Token::RightParen]);
    assert_eq!(
        lex("(\"ab\")"),
        vec![Token::LeftParen, Token::Str("ab".into()), Token::RightParen]
    );
    assert_eq!(
        lex("1.5+.5"),
        vec![Token::Float(1.5), Token::Plus, Token::Float(0.5)]
    );
    assert_eq!(lex("x`a"), vec![Token::Symbol("x".into()), sym("a")]);
    assert_eq!(
        lex("`a[1]"),
        vec![
            sym("a"),
            Token::LeftBracket,
            Token::Integer(1),
            Token::RightBracket
        ]
    );
}

#[test]
fn lex_lists_minus_never_folds_into_a_boolean() {
    assert_eq!(lex("-1b"), vec![Token::Minus, Token::Boolean(true)]);
    assert_eq!(lex("-0b"), vec![Token::Minus, Token::Boolean(false)]);
    assert_eq!(
        lex("-101b"),
        vec![Token::Minus, Token::BoolList(vec![true, false, true])]
    );
    assert_eq!(
        lex("-10b"),
        vec![Token::Minus, Token::BoolList(vec![true, false])]
    );
    assert_eq!(
        lex("2 -1b"),
        vec![Token::Integer(2), Token::Minus, Token::Boolean(true)]
    );
    // still a number when no boolean follows
    assert_eq!(
        lex("-1bc"),
        vec![Token::Integer(-1), Token::Symbol("bc".into())]
    );
    assert_eq!(lex("-12"), vec![Token::Integer(-12)]);
}

#[test]
fn lex_lists_numbers_glued_to_b_are_invalid() {
    for (src, lit) in [
        ("1.0b", "1.0"),
        ("1e3b", "1e3"),
        ("1.b", "1."),
        ("2b", "2"),
        ("102b", "102"),
        ("-2b", "2"),
    ] {
        assert_eq!(
            lex_err(src),
            parse(&format!("invalid literal: {lit}b...")),
            "{src}"
        );
    }
}

#[test]
fn lex_lists_symbol_names_are_ascii() {
    let msg = "invalid symbol: non-ASCII character 'é'; symbol names are ASCII letters, digits, '_' and '.'";
    assert_eq!(lex_err("`é"), parse(msg));
    assert_eq!(lex_err("`aé"), parse(msg));
    assert_eq!(lex_err("`a`é"), parse(msg));
    // `_` is a valid name character, not an identifier start error
    assert_eq!(lex("`_a"), vec![sym("_a")]);
}

//! `\ooo` (exactly three octal digits, at most `\377`) is one character in character and string
//! literals, and characters that are not printable ASCII display the same way.
use oxidedb::language::lexer::Token;
use oxidedb::{Atom, Column, Interpreter, Lexer};

fn lex(src: &str) -> Vec<Token> {
    Lexer::new(src).tokenize().unwrap()
}

fn lex_err(src: &str) -> String {
    Lexer::new(src).tokenize().unwrap_err().to_string()
}

fn show(src: &str) -> String {
    Interpreter::new()
        .eval_line(src)
        .unwrap()
        .unwrap()
        .to_string()
}

#[test]
fn octal_escape_is_one_character() {
    assert_eq!(lex(r#""\101""#), vec![Token::Character('A'), Token::Eof]);
    assert_eq!(lex(r#""\000""#), vec![Token::Character('\0'), Token::Eof]);
    assert_eq!(
        lex(r#""\377""#),
        vec![Token::Character('\u{ff}'), Token::Eof]
    );
    assert_eq!(lex(r#""\012""#), vec![Token::Character('\n'), Token::Eof]);
}

#[test]
fn octal_escapes_inside_strings() {
    assert_eq!(
        lex(r#""\101\102""#),
        vec![Token::Str("AB".into()), Token::Eof]
    );
    assert_eq!(
        lex(r#""x\101y""#),
        vec![Token::Str("xAy".into()), Token::Eof]
    );
    // exactly three digits are consumed; a fourth stays an ordinary character
    assert_eq!(lex(r#""\1012""#), vec![Token::Str("A2".into()), Token::Eof]);
    assert_eq!(
        lex(r#""\0001""#),
        vec![Token::Str("\u{0}1".into()), Token::Eof]
    );
}

#[test]
fn octal_escape_values_match_the_spelled_character() {
    assert_eq!(show(r#""\101"="A""#), "1b");
    assert_eq!(show(r#""\101\102""#), "\"AB\"");
}

#[test]
fn bad_octal_escapes_are_errors() {
    for src in [
        r#""\400""#,
        r#""\777""#,
        r#""\1""#,
        r#""\0""#,
        r#""\12""#,
        r#""\18""#,
        r#""\8""#,
        r#""\9""#,
        r#""a\128""#,
        r#""\"#,
    ] {
        let e = Lexer::new(src).tokenize();
        assert!(e.is_err(), "{src} lexed: {e:?}");
    }
    assert_eq!(
        lex_err(r#""\400""#),
        "'parse: invalid escape: \\400 in string"
    );
    assert_eq!(lex_err(r#""\1""#), "'parse: invalid escape: \\1 in string");
    assert_eq!(
        lex_err(r#""\12""#),
        "'parse: invalid escape: \\12 in string"
    );
    // the message names the octal digits seen, not a trailing 8
    assert_eq!(lex_err(r#""\18""#), "'parse: invalid escape: \\1 in string");
    assert_eq!(
        lex_err(r#""\128""#),
        "'parse: invalid escape: \\12 in string"
    );
    assert_eq!(lex_err(r#""\8""#), "'parse: invalid escape: \\8 in string");
}

#[test]
fn control_and_delete_characters_display_as_octal() {
    assert_eq!(show(r#""\001""#), r#""\001""#);
    assert_eq!(show(r#""\000""#), r#""\000""#);
    assert_eq!(show(r#""\033""#), r#""\033""#);
    assert_eq!(show(r#""\177""#), r#""\177""#);
    assert_eq!(show(r#""A\001B""#), r#""A\001B""#);
    assert_eq!(show(r#""\007\010\013\014""#), r#""\007\010\013\014""#);
}

#[test]
fn named_escapes_keep_their_names() {
    assert_eq!(show(r#""\011\012\015""#), r#""\t\n\r""#);
    assert_eq!(show(r#""\042\134""#), r#""\"\\""#);
    assert_eq!(show(r#""\n""#), r#""\n""#);
}

#[test]
fn printable_ascii_displays_raw() {
    assert_eq!(show(r#""\040\101\176""#), "\" A~\"");
}

#[test]
fn c1_controls_display_as_octal_and_other_latin1_raw() {
    assert_eq!(show(r#""\200""#), r#""\200""#);
    assert_eq!(show(r#""\237""#), r#""\237""#);
    // deviation from q: chars are Unicode scalar values, not bytes, so \351 is "é"
    assert_eq!(show(r#""\351""#), "\"é\"");
    assert_eq!(show(r#""\240""#), "\"\u{a0}\"");
}

#[test]
fn every_character_round_trips_through_display() {
    for code in 0u32..=255 {
        let c = char::from_u32(code).unwrap();
        let atom = Atom::Character(c).to_string();
        assert_eq!(
            lex(&atom),
            vec![Token::Character(c), Token::Eof],
            "{code}: {atom}"
        );

        let col = Column::Char(vec![c, 'a', c]).to_string();
        assert_eq!(
            lex(&col),
            vec![Token::Str(format!("{c}a{c}")), Token::Eof],
            "{code}: {col}"
        );
    }
}

#[test]
fn token_display_matches_value_display() {
    assert_eq!(Token::Character('\u{1}').to_string(), r#""\001""#);
    assert_eq!(Token::Str("a\u{1}\"".into()).to_string(), r#""a\001\"""#);
}

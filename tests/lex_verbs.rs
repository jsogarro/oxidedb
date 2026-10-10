//! Verb and punctuation tokens (lexer only; the parser reports each as not
//! yet implemented).
use oxidedb::language::lexer::Token;
use oxidedb::{Interpreter, Lexer, QError};

fn lex(src: &str) -> Vec<Token> {
    let mut tokens = Lexer::new(src).tokenize().unwrap();
    assert_eq!(tokens.pop(), Some(Token::Eof));
    tokens
}

fn eval_err(src: &str) -> QError {
    Interpreter::new().eval_line(src).expect_err(src)
}

fn unexpected(ch: &str) -> QError {
    QError::Parse(format!("unexpected character: {ch}"))
}

const SINGLES: &[(&str, Token)] = &[
    ("=", Token::Equal),
    ("<", Token::Less),
    (">", Token::Greater),
    ("<>", Token::NotEqual),
    ("<=", Token::LessEqual),
    (">=", Token::GreaterEqual),
    ("#", Token::Hash),
    (",", Token::Comma),
    ("!", Token::Bang),
    ("{", Token::LeftBrace),
    ("}", Token::RightBrace),
    ("$", Token::Dollar),
    ("@", Token::At),
    ("'", Token::Quote),
    ("':", Token::EachPrior),
];

#[test]
fn lex_verbs_comparisons() {
    assert_eq!(lex("<>"), vec![Token::NotEqual]);
    assert_eq!(lex("<="), vec![Token::LessEqual]);
    assert_eq!(lex(">="), vec![Token::GreaterEqual]);
    assert_eq!(lex("<"), vec![Token::Less]);
    assert_eq!(lex(">"), vec![Token::Greater]);
    assert_eq!(lex("="), vec![Token::Equal]);
    // spaces split multi-character operators
    assert_eq!(lex("< >"), vec![Token::Less, Token::Greater]);
    assert_eq!(lex("< ="), vec![Token::Less, Token::Equal]);
    assert_eq!(lex("> ="), vec![Token::Greater, Token::Equal]);
    // longest match, left to right
    assert_eq!(lex("<=>"), vec![Token::LessEqual, Token::Greater]);
    assert_eq!(lex("<<"), vec![Token::Less, Token::Less]);
    assert_eq!(lex("=="), vec![Token::Equal, Token::Equal]);
    assert_eq!(lex("><"), vec![Token::Greater, Token::Less]);
    assert_eq!(
        lex("1<=2"),
        vec![Token::Integer(1), Token::LessEqual, Token::Integer(2)]
    );
}

#[test]
fn lex_verbs_take_join_bang() {
    assert_eq!(lex("#"), vec![Token::Hash]);
    assert_eq!(lex(","), vec![Token::Comma]);
    assert_eq!(lex("!"), vec![Token::Bang]);
    assert_eq!(
        lex("2#3"),
        vec![Token::Integer(2), Token::Hash, Token::Integer(3)]
    );
    assert_eq!(
        lex("1,2"),
        vec![Token::Integer(1), Token::Comma, Token::Integer(2)]
    );
    assert_eq!(lex("!3"), vec![Token::Bang, Token::Integer(3)]);
}

#[test]
fn lex_verbs_braces_dollar_at() {
    assert_eq!(lex("{"), vec![Token::LeftBrace]);
    assert_eq!(lex("}"), vec![Token::RightBrace]);
    assert_eq!(lex("$"), vec![Token::Dollar]);
    assert_eq!(lex("@"), vec![Token::At]);
    assert_eq!(
        lex("{x+1}"),
        vec![
            Token::LeftBrace,
            Token::Symbol("x".into()),
            Token::Plus,
            Token::Integer(1),
            Token::RightBrace
        ]
    );
}

#[test]
fn lex_verbs_quote() {
    assert_eq!(lex("'"), vec![Token::Quote]);
    assert_eq!(lex("+'"), vec![Token::Plus, Token::Quote]);
    assert_eq!(lex("':"), vec![Token::EachPrior]);
    // a quote and a colon with a space between are two tokens
    assert_eq!(lex("' :"), vec![Token::Quote, Token::Colon]);
    assert_eq!(lex("+':"), vec![Token::Plus, Token::EachPrior]);
}

#[test]
fn lex_verbs_adverbs_glued() {
    assert_eq!(lex("+/"), vec![Token::Plus, Token::Over]);
    assert_eq!(lex("+\\"), vec![Token::Plus, Token::Scan]);
}

#[test]
fn lex_verbs_interplay() {
    // negative literals after a verb, comments after a verb
    assert_eq!(lex("<-1"), vec![Token::Less, Token::Integer(-1)]);
    assert_eq!(lex(",-1"), vec![Token::Comma, Token::Integer(-1)]);
    assert_eq!(lex("= / c"), vec![Token::Equal]);
    assert_eq!(lex("=/"), vec![Token::Equal, Token::Over]);
    assert_eq!(
        lex("`a,`b"),
        vec![Token::Sym("a".into()), Token::Comma, Token::Sym("b".into())]
    );
}

#[test]
fn lex_verbs_unknown_characters_still_rejected() {
    for ch in ["^", "&", "|", "~", "?", "&&"] {
        let first = &ch[..1];
        assert_eq!(
            Lexer::new(ch).tokenize().expect_err(ch),
            unexpected(first),
            "{ch}"
        );
    }
    assert_eq!(
        Lexer::new("_").tokenize().unwrap_err(),
        QError::parse("invalid identifier: a name must start with a letter, not '_'")
    );
}

#[test]
fn lex_verbs_parser_rejects_as_nyi() {
    for (src, _) in SINGLES {
        for expr in [src.to_string(), format!("1{src}2"), format!("1 {src}")] {
            let err = eval_err(&expr);
            assert_eq!(err, QError::Nyi(src.to_string()), "{expr}");
        }
    }
    assert_eq!(eval_err("=").to_string(), "'nyi: =");
    assert_eq!(eval_err("1<>2").to_string(), "'nyi: <>");
}

#[test]
fn lex_verbs_display_source_form() {
    for (src, token) in SINGLES {
        assert_eq!(token.to_string(), *src);
        assert_eq!(lex(src), vec![token.clone()], "{src}");
    }
}

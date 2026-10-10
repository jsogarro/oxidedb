//! Verb and punctuation tokens (lexer only; the parser reports each as not
//! yet implemented; the dyadic verbs parse since `tests/dyadic_verbs.rs`).
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

const DYADIC: &[&str] = &["=", "<", ">", "<>", "<=", ">=", "#", ",", "!"];

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
    ("/:", Token::EachRight),
    ("\\:", Token::EachLeft),
    ("::", Token::DoubleColon),
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
    // The dyadic verbs now parse (tests/dyadic_verbs.rs); only the rest are still nyi here.
    for (src, _) in SINGLES.iter().filter(|(s, _)| !DYADIC.contains(s)) {
        // after whitespace (or at line start) a `/` starts a comment, so glue it to a `1`
        for expr in [format!("1{src}"), format!("1{src}2"), format!("(1{src}")] {
            let err = eval_err(&expr);
            let detail = match *src {
                "'" => "adverb ' (each)".to_string(),
                "':" => "adverb ': (each-prior)".to_string(),
                "/:" => "adverb /: (each-right)".to_string(),
                "\\:" => "adverb \\: (each-left)".to_string(),
                s => s.to_string(),
            };
            assert_eq!(err, QError::Nyi(detail), "{expr}");
        }
    }
    for src in ["=", "<", "{", "}", "'", "::"] {
        let want = match src {
            "'" => "adverb ' (each)",
            s => s,
        };
        assert_eq!(eval_err(src), QError::Nyi(want.into()), "{src}");
    }
    assert_eq!(eval_err("=").to_string(), "'nyi: =");
    assert_eq!(eval_err("<>").to_string(), "'nyi: <>");
    assert_eq!(eval_err("1$2").to_string(), "'nyi: $");
}

#[test]
fn lex_verbs_display_source_form() {
    for (src, token) in SINGLES {
        assert_eq!(token.to_string(), *src);
        if !src.starts_with('/') {
            assert_eq!(lex(src), vec![token.clone()], "{src}");
        }
        // glued to a verb, so `/` and `\\` are not read as a comment or scan
        assert_eq!(
            lex(&format!("+{src}")),
            vec![Token::Plus, token.clone()],
            "+{src}"
        );
    }
}

#[test]
fn lex_verbs_greater_greater_is_two_tokens() {
    assert_eq!(lex(">>"), vec![Token::Greater, Token::Greater]);
    assert_eq!(lex(">>="), vec![Token::Greater, Token::GreaterEqual]);
    assert_eq!(lex("<>>"), vec![Token::NotEqual, Token::Greater]);
}

#[test]
fn lex_verbs_each_right_left_double_colon() {
    assert_eq!(lex("+/:"), vec![Token::Plus, Token::EachRight]);
    assert_eq!(lex("+\\:"), vec![Token::Plus, Token::EachLeft]);
    assert_eq!(
        lex("1/:2"),
        vec![Token::Integer(1), Token::EachRight, Token::Integer(2)]
    );
    assert_eq!(
        lex("x::1"),
        vec![
            Token::Symbol("x".into()),
            Token::DoubleColon,
            Token::Integer(1)
        ]
    );
    assert_eq!(lex(":::"), vec![Token::DoubleColon, Token::Colon]);
    assert_eq!(lex(": :"), vec![Token::Colon, Token::Colon]);
    // a space splits them: `/` after space is a comment, `\` is a scan
    assert_eq!(lex("+/ :"), vec![Token::Plus, Token::Over, Token::Colon]);
    assert_eq!(lex("+\\ :"), vec![Token::Plus, Token::Scan, Token::Colon]);
    // q: ` /` starts a comment, so a spaced `/:` is one
    assert_eq!(lex("1 /: 2"), vec![Token::Integer(1)]);
    assert_eq!(lex("/:"), vec![]);
}

/// Glued `-digit` after anything but a noun ending is a literal.
#[test]
fn lex_verbs_negative_literal_after_every_non_noun() {
    for (src, token) in SINGLES {
        if *token == Token::RightBrace {
            continue;
        }
        let want = vec![Token::Integer(1), token.clone(), Token::Integer(-1)];
        assert_eq!(lex(&format!("1{src}-1")), want, "{src}");
        assert_eq!(lex(&format!("1{src} -1")), want, "{src}");
    }
    assert_eq!(lex("{-1"), vec![Token::LeftBrace, Token::Integer(-1)]);
}

/// After a noun-ending token a glued `-` is subtraction; a spaced one is a literal.
#[test]
fn lex_verbs_negative_literal_after_noun_endings() {
    for (close, token) in [
        (")", Token::RightParen),
        ("]", Token::RightBracket),
        ("}", Token::RightBrace),
    ] {
        assert_eq!(
            lex(&format!("{close}-1")),
            vec![token.clone(), Token::Minus, Token::Integer(1)],
            "{close}"
        );
        assert_eq!(
            lex(&format!("{close} -1")),
            vec![token, Token::Integer(-1)],
            "{close}"
        );
    }
    assert_eq!(
        lex("{x}-1"),
        vec![
            Token::LeftBrace,
            Token::Symbol("x".into()),
            Token::RightBrace,
            Token::Minus,
            Token::Integer(1)
        ]
    );
}

#[test]
fn lex_verbs_nyi_inside_parentheses() {
    for (src, detail) in [
        ("(1$2)", "$"),
        ("x:(1@2)", "@"),
        ("(1/2)", "adverb '/'"),
        ("(1\\2)", "adverb '\\'"),
        ("(1'2)", "adverb ' (each)"),
        ("(1/:2)", "adverb /: (each-right)"),
    ] {
        assert_eq!(eval_err(src), QError::Nyi(detail.into()), "{src}");
    }
    // anything else still reports the missing parenthesis
    for src in ["(1", "(1 2", "(1+"] {
        assert!(matches!(eval_err(src), QError::Parse(_)), "{src}");
    }
    assert_eq!(
        eval_err("(1 2").to_string(),
        "'parse: expected ')' after expression"
    );
}

#[test]
fn lex_verbs_adverb_nyi_details_are_readable() {
    assert_eq!(eval_err("1'2").to_string(), "'nyi: adverb ' (each)");
    assert_eq!(eval_err("1':2").to_string(), "'nyi: adverb ': (each-prior)");
    assert_eq!(eval_err("1/:2").to_string(), "'nyi: adverb /: (each-right)");
    assert_eq!(
        eval_err("1\\:2").to_string(),
        "'nyi: adverb \\: (each-left)"
    );
    assert_eq!(eval_err("1::2").to_string(), "'nyi: ::");
    assert_eq!(eval_err("6/2").to_string(), "'nyi: adverb '/'");
}

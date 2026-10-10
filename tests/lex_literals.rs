use oxidedb::language::lexer::Token;
use oxidedb::{Atom, Interpreter, Lexer, Parser, Value};

fn lex(src: &str) -> Vec<Token> {
    let mut tokens = Lexer::new(src).tokenize().unwrap();
    assert_eq!(tokens.pop(), Some(Token::Eof));
    tokens
}

fn eval(interp: &mut Interpreter, src: &str) -> Value {
    let tokens = Lexer::new(src).tokenize().unwrap();
    let ast = Parser::new(tokens).parse().unwrap();
    interp.evaluate(ast).unwrap()
}

#[test]
fn boolean_literals() {
    let mut interp = Interpreter::new();
    assert_eq!(eval(&mut interp, "1b"), Atom::Boolean(true));
    assert_eq!(eval(&mut interp, "0b"), Atom::Boolean(false));
    eval(&mut interp, "flag:1b");
    assert_eq!(eval(&mut interp, "flag"), Atom::Boolean(true));
}

#[test]
fn lex_neg_literal_at_start() {
    assert_eq!(lex("-5"), vec![Token::Integer(-5)]);
}

#[test]
fn lex_neg_after_operator() {
    assert_eq!(
        lex("3*-2"),
        vec![Token::Integer(3), Token::Multiply, Token::Integer(-2)]
    );
}

#[test]
fn lex_minus_glued_is_operator() {
    let expect = |a| vec![a, Token::Minus, Token::Integer(1)];
    assert_eq!(lex("2-1"), expect(Token::Integer(2)));
    assert_eq!(lex("x-1"), expect(Token::Symbol("x".into())));
}

#[test]
fn lex_minus_spaced_is_operator() {
    assert_eq!(
        lex("2 - 1"),
        vec![Token::Integer(2), Token::Minus, Token::Integer(1)]
    );
}

#[test]
fn lex_neg_literal_min_long() {
    assert_eq!(lex("-9223372036854775808"), vec![Token::Integer(i64::MIN)]);
}

#[test]
fn lex_neg_after_space_following_noun() {
    assert_eq!(lex("2 -1"), vec![Token::Integer(2), Token::Integer(-1)]);
}

#[test]
fn neg_literal_keeps_arithmetic() {
    assert_eq!(eval(&mut Interpreter::new(), "-5 + 3"), Atom::Integer(-2));
}

#[test]
fn lex_digit_then_b_identifier_is_not_boolean() {
    let sym = |s: &str| Token::Symbol(s.into());
    // `10b` is a boolean vector now (see tests/lex_lists.rs).
    assert_eq!(lex("10b"), vec![Token::BoolList(vec![true, false])]);
    assert_eq!(lex("0b1"), vec![Token::Integer(0), sym("b1")]);
    assert_eq!(lex("1bc"), vec![Token::Integer(1), sym("bc")]);
    assert_eq!(lex("1b_"), vec![Token::Integer(1), sym("b_")]);
}

#[test]
fn lex_neg_after_each_noun_kind() {
    let neg = Token::Integer(-1);
    assert_eq!(lex("1b -1"), vec![Token::Boolean(true), neg.clone()]);
    assert_eq!(lex("\"a\" -1"), vec![Token::Character('a'), neg.clone()]);
    assert_eq!(
        lex("(1) -1"),
        vec![
            Token::LeftParen,
            Token::Integer(1),
            Token::RightParen,
            neg.clone()
        ]
    );
    assert_eq!(
        lex("x[0] -1"),
        vec![
            Token::Symbol("x".into()),
            Token::LeftBracket,
            Token::Integer(0),
            Token::RightBracket,
            neg
        ]
    );
}

#[test]
fn lex_minus_glued_after_each_noun_kind_is_operator() {
    for (src, noun_len) in [("1b-1", 1), ("\"a\"-1", 1), ("(1)-1", 3), ("x[0]-1", 4)] {
        let tokens = lex(src);
        assert_eq!(tokens[noun_len], Token::Minus, "{src}");
        assert_eq!(tokens[noun_len + 1], Token::Integer(1), "{src}");
    }
}

use oxidedb::language::lexer::Token;
use oxidedb::{Atom, Interpreter, Lexer, Parser};

fn lex(src: &str) -> Vec<Token> {
    let mut tokens = Lexer::new(src).tokenize().unwrap();
    assert_eq!(tokens.pop(), Some(Token::Eof));
    tokens
}

fn eval(interp: &mut Interpreter, src: &str) -> Atom {
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

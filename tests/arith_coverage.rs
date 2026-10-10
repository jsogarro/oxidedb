use oxidedb::{Atom, Interpreter, Lexer, Parser};

fn eval(src: &str) -> anyhow::Result<Atom> {
    let tokens = Lexer::new(src).tokenize()?;
    let ast = Parser::new(tokens).parse()?;
    Interpreter::new().evaluate(ast)
}

#[test]
fn negate_float_flips_sign() {
    assert_eq!(eval("-(2.5)").unwrap(), Atom::Float(-2.5));
    assert_eq!(eval("-(-(2.5))").unwrap(), Atom::Float(2.5));
    assert_eq!(eval("-(1.5 + 1)").unwrap(), Atom::Float(-2.5));
}

#[test]
fn negate_integer_flips_sign() {
    assert_eq!(eval("-(2)").unwrap(), Atom::Integer(-2));
    assert_eq!(eval("-(-(2))").unwrap(), Atom::Integer(2));
    assert_eq!(eval("-(1 + 2)").unwrap(), Atom::Integer(-3));
}

#[test]
fn negate_literal_matches_negate_expression() {
    assert_eq!(eval("-2.5").unwrap(), Atom::Float(-2.5));
    assert_eq!(eval("-2").unwrap(), Atom::Integer(-2));
}

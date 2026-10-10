use oxidedb::language::lexer::Token;
use oxidedb::{Atom, Interpreter, Lexer, Parser};

const N: usize = 200_000;

fn run(interp: &mut Interpreter, src: &str) -> anyhow::Result<Atom> {
    let tokens = Lexer::new(src).tokenize()?;
    let ast = Parser::new(tokens).parse()?;
    interp.evaluate(ast)
}

fn eval(src: &str) -> anyhow::Result<Atom> {
    run(&mut Interpreter::new(), src)
}

/// Runs `f` on a thread with a small explicit stack, so unbounded recursion shows up quickly.
fn small_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .expect("thread panicked")
}

#[test]
fn deep_nesting_returns_error_not_abort() {
    let parens = format!("{}1{}", "(".repeat(N), ")".repeat(N));
    let terms = format!("{}1", "1+".repeat(N));
    let minuses = format!("{}1", "- ".repeat(N));
    for src in [parens, terms, minuses] {
        let err = small_stack(move || eval(&src)).unwrap_err().to_string();
        assert!(err.contains("nested too deeply"), "got: {err}");
    }
    // Within the limit still evaluates (right-nested chain, 200 deep).
    let ok = format!("{}1", "1+".repeat(200));
    assert_eq!(small_stack(move || eval(&ok)).unwrap(), Atom::Integer(201));
    // Depth is released on the way out: many sibling parens are not nesting.
    let siblings = format!("{}1", "(1)+".repeat(150));
    assert_eq!(eval(&siblings).unwrap(), Atom::Integer(151));
}

#[test]
fn deep_nesting_monadic_minus_whole_rhs() {
    let mut i = Interpreter::new();
    run(&mut i, "x:2").unwrap();
    assert_eq!(run(&mut i, "-x+3").unwrap(), Atom::Integer(-5));
    assert_eq!(eval("-(3)+2").unwrap(), Atom::Integer(-5));
}

#[test]
fn deep_nesting_neg_literal_unchanged() {
    assert_eq!(eval("-5 + 3").unwrap(), Atom::Integer(-2));
}

#[test]
fn parser_new_without_eof_does_not_panic() {
    for tokens in [vec![], vec![Token::Integer(1)]] {
        let r = std::panic::catch_unwind(|| Parser::new(tokens).parse());
        assert!(r.is_ok(), "Parser::new panicked without trailing Eof");
    }
}

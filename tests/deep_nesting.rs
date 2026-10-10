use oxidedb::language::lexer::Token;
use oxidedb::{Atom, Interpreter, Lexer, Parser};

const N: usize = 200_000;

fn run(interp: &mut Interpreter, src: &str) -> anyhow::Result<Atom> {
    let tokens = Lexer::new(src).tokenize()?;
    let ast = Parser::new(tokens).parse()?;
    Ok(interp.evaluate(ast)?)
}

fn eval(src: &str) -> anyhow::Result<Atom> {
    run(&mut Interpreter::new(), src)
}

/// Runs `f` on a thread with a generous explicit stack, so a deep-recursion regression
/// fails here rather than depending on the harness thread size.
fn big_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .expect("thread panicked")
}

fn nested_err(src: String) -> String {
    big_stack(move || eval(&src)).unwrap_err().to_string()
}

#[test]
fn deep_nesting_returns_error_not_abort() {
    let parens = format!("{}1{}", "(".repeat(N), ")".repeat(N));
    let minuses = format!("{}1", "- ".repeat(N));
    for src in [parens, minuses] {
        assert!(nested_err(src).contains("nested too deeply"));
    }
    // A flat chain of 200,000 terms is a clean error too, never an abort.
    assert!(nested_err(format!("{}1", "1+".repeat(N))).contains("too long"));
}

#[test]
fn deep_nesting_limit_boundary() {
    // Nesting limit: 127 levels ok, 128 errors.
    let parens = |n: usize| format!("{}1{}", "(".repeat(n), ")".repeat(n));
    let minuses = |n: usize| format!("{}1", "- ".repeat(n));
    assert!(big_stack(move || eval(&parens(127))).is_ok());
    assert!(nested_err(parens(128)).contains("nested too deeply"));
    assert!(big_stack(move || eval(&minuses(127))).is_ok());
    assert!(nested_err(minuses(128)).contains("nested too deeply"));
    // Chain limit: 1,999 operators ok, 2,000 errors.
    let chain = |ops: usize| format!("{}1", "1+".repeat(ops));
    assert_eq!(
        big_stack(move || eval(&chain(1999))).unwrap(),
        Atom::Integer(2000)
    );
    assert!(nested_err(chain(2000)).contains("expression too long"));
}

#[test]
fn flat_expressions_are_not_nesting() {
    let flat = format!("{}1", "1+".repeat(999));
    assert_eq!(big_stack(move || eval(&flat)).unwrap(), Atom::Integer(1000));
    let siblings = format!("{}1", "(1)+".repeat(300));
    assert_eq!(
        big_stack(move || eval(&siblings)).unwrap(),
        Atom::Integer(301)
    );
    // Assignment in operand position keeps its meaning: x + (x:2) with RHS first.
    let mut i = Interpreter::new();
    run(&mut i, "x:1").unwrap();
    assert_eq!(run(&mut i, "x+x:2").unwrap(), Atom::Integer(4));
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
    assert_eq!(eval("- 5 + 3").unwrap(), Atom::Integer(-8));
}

#[test]
fn parser_new_without_eof_does_not_panic() {
    for tokens in [vec![], vec![Token::Integer(1)]] {
        let r = std::panic::catch_unwind(|| Parser::new(tokens).parse());
        assert!(r.is_ok(), "Parser::new panicked without trailing Eof");
    }
}

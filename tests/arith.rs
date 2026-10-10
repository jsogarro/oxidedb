use oxidedb::{Atom, Interpreter, Lexer, Parser};

fn run(interp: &mut Interpreter, src: &str) -> anyhow::Result<Atom> {
    let tokens = Lexer::new(src).tokenize()?;
    let ast = Parser::new(tokens).parse()?;
    Ok(interp.evaluate(ast)?)
}

fn eval(src: &str) -> anyhow::Result<Atom> {
    run(&mut Interpreter::new(), src)
}

/// Runs `src`, turning a panic into an `Err` so a bug shows as a test failure message.
fn eval_no_panic(src: &str) -> anyhow::Result<Atom> {
    let owned = src.to_string();
    std::thread::spawn(move || eval(&owned))
        .join()
        .unwrap_or_else(|_| Err(anyhow::anyhow!("PANIC evaluating {src}")))
}

fn float(src: &str) -> f64 {
    match eval(src).unwrap() {
        Atom::Float(f) => f,
        other => panic!("{src}: expected Float, got {other:?}"),
    }
}

#[test]
fn arith_rhs_evaluated_first() {
    let mut i = Interpreter::new();
    run(&mut i, "x:1").unwrap();
    assert_eq!(run(&mut i, "x+x:2").unwrap(), Atom::Integer(4));
}

#[test]
fn integer_overflow_is_error() {
    for src in [
        "9223372036854775807+1",
        "9223372036854775807+2",
        "9223372036854775807*2",
        "(0-9223372036854775807)-2",
        // exactly i64::MIN is reserved for the long null, so it is also an overflow
        "(0-9223372036854775807)-1",
    ] {
        let err = eval_no_panic(src).expect_err(src);
        assert!(err.to_string().contains("overflow"), "{src}: {err}");
    }
    assert_eq!(
        eval("9223372036854775806+1").unwrap(),
        Atom::Integer(i64::MAX)
    );
}

#[test]
fn divide_always_float() {
    assert_eq!(eval("15%3").unwrap(), Atom::Float(5.0));
    assert_eq!(eval("7%2").unwrap(), Atom::Float(3.5));
    assert_eq!(eval("8%2%2").unwrap(), Atom::Float(8.0));
    assert_eq!(eval("10%4.0").unwrap(), Atom::Float(2.5));
}

#[test]
fn arith_divide_by_zero_is_ieee() {
    assert_eq!(float("1%0"), f64::INFINITY);
    assert_eq!(float("(0-1)%0"), f64::NEG_INFINITY);
    assert!(float("0%0").is_nan());
    assert_eq!(float("1.0%0.0"), f64::INFINITY);
}

#[test]
fn arith_float_ops() {
    assert_eq!(eval("2.5*2").unwrap(), Atom::Float(5.0));
    assert_eq!(eval("-2.5").unwrap(), Atom::Float(-2.5));
    assert_eq!(eval("5.0-7.5").unwrap(), Atom::Float(-2.5));
}

use oxidedb::{Interpreter, Lexer, Parser};

fn eval(src: &str) -> anyhow::Result<oxidedb::Atom> {
    // No statement separator yet: `;`-split lines share one interpreter.
    let mut interp = Interpreter::new();
    let mut last = None;
    for stmt in src.split(';') {
        let tokens = Lexer::new(stmt).tokenize()?;
        last = Some(interp.evaluate(Parser::new(tokens).parse()?)?);
    }
    Ok(last.unwrap())
}

fn show(src: &str) -> String {
    match eval(src) {
        Ok(a) => a.to_string(),
        Err(e) => panic!("{src:?} failed: {e}"),
    }
}

fn err(src: &str) -> String {
    match eval(src) {
        Ok(a) => panic!("{src:?} should fail, got {a}"),
        Err(e) => e.to_string(),
    }
}

fn lex_err(src: &str) -> String {
    Lexer::new(src).tokenize().unwrap_err().to_string()
}

fn type_code(src: &str) -> i8 {
    eval(src).unwrap().type_code()
}

// ---- #30 float exponents ----

#[test]
fn exponent_forms() {
    assert_eq!(show("1e3"), "1000f");
    assert_eq!(show("1.5e3"), "1500f");
    assert_eq!(show("1e-3"), "0.001");
    assert_eq!(show("1e+3"), "1000f");
    assert_eq!(show("-1e3"), "-1000f");
    assert_eq!(show(".5e1"), "5f");
    assert_eq!(show("2e0"), "2f");
    assert_eq!(type_code("1e3"), -9);
}

#[test]
fn exponent_is_lowercase_only() {
    assert!(eval("1E3").is_err());
}

#[test]
fn exponent_without_digits_is_error() {
    for src in ["1e", "1e+", "1e-", "1.5e", "1e+ 3"] {
        assert!(lex_err(src).contains("exponent"), "{src}");
    }
}

#[test]
fn exponent_in_expressions() {
    assert_eq!(show("1e3+1"), "1001f");
    assert_eq!(show("2-1e1"), "-8f");
}

#[test]
fn out_of_range_float_is_error() {
    assert!(lex_err("1e999").contains("Float out of range"));
    assert!(lex_err("-1e999").contains("Float out of range"));
    let big = format!("{}.0", "9".repeat(400));
    assert!(lex_err(&big).contains("Float out of range"));
    assert_eq!(show("1.7e308"), "1.7e+308");
}

// ---- float suffix ----

#[test]
fn float_suffix() {
    assert_eq!(type_code("1f"), -9);
    assert_eq!(show("1f"), "1f");
    assert_eq!(show("2.5f"), "2.5");
    assert_eq!(show("-3f"), "-3f");
    assert_eq!(show("1e3f"), "1000f");
    assert_eq!(show("1f+1"), "2f");
}

#[test]
fn float_suffix_needs_boundary() {
    let toks = Lexer::new("1foo").tokenize().unwrap();
    assert_eq!(toks[0].to_string(), "1");
    assert_eq!(toks[1].to_string(), "foo");
    // `1foo` is not a float literal: the number ends and `foo` is a name.
    assert!(eval("1foo").is_err());
    assert!(eval("x:1;1fx").is_err());
}

// ---- null and infinity literals ----

#[test]
fn null_and_infinity_literals_round_trip() {
    for (src, out) in [("0N", "0N"), ("0n", "0n"), ("0w", "0w"), ("-0w", "-0w")] {
        assert_eq!(show(src), out);
    }
    assert_eq!(type_code("0N"), -7);
    assert_eq!(type_code("0n"), -9);
    assert_eq!(type_code("0w"), -9);
    assert_eq!(type_code("-0w"), -9);
}

#[test]
fn infinities_and_nulls_have_expected_values() {
    assert_eq!(eval("0w").unwrap().as_float(), Some(f64::INFINITY));
    assert_eq!(eval("-0w").unwrap().as_float(), Some(f64::NEG_INFINITY));
    assert!(eval("0n").unwrap().is_null());
    assert!(eval("0N").unwrap().is_null());
    assert_eq!(eval("0N").unwrap().as_integer(), Some(i64::MIN));
}

#[test]
fn infinity_arithmetic() {
    assert_eq!(show("0w+1"), "0w");
    assert_eq!(show("0w-0w"), "0n");
    assert_eq!(show("-(0w)"), "-0w");
}

#[test]
fn long_infinity_is_clean_error() {
    assert!(lex_err("0W").contains("not supported"));
    assert!(lex_err("-0W").contains("not supported"));
}

#[test]
fn malformed_null_literals_are_errors() {
    for src in ["0Nd", "0nx", "0wx", "0Nx", "0N_", "0n1"] {
        assert!(lex_err(src).contains("Invalid literal"), "{src}");
    }
}

#[test]
fn null_letters_only_after_a_lone_zero() {
    // `10N` / `1n` / `0.0n` are not null literals.
    for src in ["10N", "1n", "1w", "0.0n", "00N"] {
        assert!(eval(src).is_err(), "{src}");
    }
}

#[test]
fn spaced_zero_then_name_is_unchanged() {
    assert_eq!(show("w:5;0 +w"), "5");
}

// ---- #61 null semantics ----

#[test]
fn min_literal_is_long_null() {
    assert_eq!(show("-9223372036854775808"), "0N");
    assert_eq!(show("m:-9223372036854775808;m"), "0N");
    assert_eq!(show("-9223372036854775808+0"), "0N");
}

#[test]
fn long_null_propagates() {
    for (src, out) in [
        ("0N+1", "0N"),
        ("1+0N", "0N"),
        ("0N-1", "0N"),
        ("1-0N", "0N"),
        ("0N*2", "0N"),
        ("2*0N", "0N"),
        ("0N+0N", "0N"),
        ("-0N", "0N"),
        ("-(0N)", "0N"),
        ("0N*0", "0N"),
        ("0*0N", "0N"),
    ] {
        assert_eq!(show(src), out, "{src}");
    }
}

#[test]
fn null_with_float_is_float_null() {
    for src in ["0N+1.5", "1.5+0N", "0N*2f", "2f-0N", "0N+0n", "0n+0N"] {
        assert_eq!(show(src), "0n", "{src}");
    }
    assert_eq!(type_code("0N+1.5"), -9);
}

#[test]
fn null_divide_is_float_null() {
    assert_eq!(show("0N%2"), "0n");
    assert_eq!(show("2%0N"), "0n");
    assert_eq!(show("0N%0N"), "0n");
    assert_eq!(show("0N%2.5"), "0n");
    assert_eq!(show("2.5%0N"), "0n");
}

#[test]
fn float_null_propagates() {
    assert_eq!(show("0n+1"), "0n");
    assert_eq!(show("1*0n"), "0n");
    assert_eq!(show("-0n"), "0n");
}

#[test]
fn non_null_overflow_to_min_is_still_error() {
    assert!(err("-9223372036854775807-1").contains("overflow"));
    assert!(err("9223372036854775807+1").contains("overflow"));
    assert!(err("-9223372036854775807*2").contains("overflow"));
    assert!(err("4611686018427387904*-2").contains("overflow"));
}

#[test]
fn negating_min_is_null() {
    assert_eq!(show("-(-9223372036854775808)"), "0N");
}

// ---- #33 identifiers ----

#[test]
fn ascii_identifiers_work() {
    assert_eq!(show("abc:1;abc"), "1");
    assert_eq!(show("a_1:2;a_1"), "2");
    assert_eq!(show("A:3;A"), "3");
    assert_eq!(show("x9:4;x9+1"), "5");
    assert_eq!(show("a__:6;a__"), "6");
}

#[test]
fn leading_underscore_is_error() {
    for src in ["_a", "_", "_1", "x:1;_x"] {
        assert!(lex_err(src).contains("must start with a letter"), "{src}");
    }
}

#[test]
fn non_ascii_identifier_chars_are_errors() {
    for src in [
        "é", "é:1", "xé", "x٣:1", "٣", "x_é", "ab٣c", "日本", "x:1;x٣",
    ] {
        assert!(lex_err(src).contains("ASCII"), "{src}");
    }
}

//! Error wording, kinds and exact output channels (review follow-ups).
use oxidedb::{Interpreter, Lexer, QError};
use std::io::Write;
use std::process::{Command, Output, Stdio};

fn lex_err(src: &str) -> QError {
    Lexer::new(src).tokenize().expect_err(src)
}

fn eval_err(src: &str) -> QError {
    Interpreter::new().eval_line(src).expect_err(src)
}

fn parse(detail: &str) -> QError {
    QError::Parse(detail.into())
}

const STRINGS: &str = "strings (a character literal holds exactly one character)";

#[test]
fn lexer_details_are_lowercase() {
    assert_eq!(
        lex_err("_a"),
        parse("invalid identifier: a name must start with a letter, not '_'")
    );
    assert_eq!(
        lex_err("é"),
        parse(
            "invalid identifier: non-ASCII character 'é'; names are ASCII letters, digits and '_'"
        )
    );
    assert_eq!(
        lex_err("1e"),
        parse("invalid float: exponent needs digits after 'e'")
    );
    assert_eq!(lex_err("1e999"), parse("float out of range: 1e999"));
    assert_eq!(lex_err("^"), parse("unexpected character: ^"));
    assert_eq!(lex_err("0Nx"), parse("invalid literal: 0N..."));
}

#[test]
fn integer_too_large_is_parse() {
    assert_eq!(
        lex_err("99999999999999999999"),
        parse("invalid integer: 99999999999999999999")
    );
}

#[test]
fn bare_quote_is_parse() {
    assert_eq!(lex_err("\""), parse("unterminated character literal"));
    assert_eq!(lex_err("\"a"), parse("unterminated character literal"));
    assert_eq!(lex_err("\"ab"), parse("unterminated character literal"));
}

#[test]
fn terminated_strings_are_not_yet_implemented() {
    for src in ["\"ab\"", "\"\"", "x:\"Alice\""] {
        assert_eq!(eval_err(src), QError::Nyi(STRINGS.into()), "{src}");
    }
    assert_eq!(eval_err("\"ab\"").to_string(), format!("'nyi: {STRINGS}"));
}

#[test]
fn triple_quote_is_an_unterminated_literal() {
    assert_eq!(lex_err("\"\"\""), parse("unterminated character literal"));
}

#[test]
fn symbols_are_not_yet_implemented() {
    assert_eq!(eval_err("`a"), QError::Nyi("symbols".into()));
    assert_eq!(eval_err("`a").to_string(), "'nyi: symbols");
}

#[test]
fn long_infinity_detail_is_concise() {
    for src in ["0W", "-0W"] {
        assert_eq!(
            lex_err(src).to_string(),
            "'nyi: 0W (long infinity)",
            "{src}"
        );
    }
}

// ---- binary output ----

fn bin() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_oxidedb"));
    c.env("NO_COLOR", "1");
    c
}

fn run_file(name: &str, bytes: &[u8]) -> Output {
    let path =
        std::env::temp_dir().join(format!("oxidedb_review_{}_{}.o", std::process::id(), name));
    std::fs::write(&path, bytes).unwrap();
    let out = bin().arg(&path).output().unwrap();
    let _ = std::fs::remove_file(&path);
    out
}

fn lines(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(String::from)
        .collect()
}

#[test]
fn file_mode_error_line_is_exact_with_file_line_number() {
    // blank and comment lines count: the failing line is line 5 of the file
    let out = run_file("lineno", b"1+2\n\n/ note\n\n1b+1\n9\n");
    assert_eq!(out.status.code(), Some(1));
    let e = lines(&out.stderr);
    assert_eq!(e.len(), 2, "stderr: {e:?}");
    assert!(e[0].contains("Executing"), "{e:?}");
    assert_eq!(e[1], "line 5: 'type");
    assert_eq!(lines(&out.stdout), ["3"]);
}

#[test]
fn file_mode_undefined_variable_has_no_double_parentheses() {
    let out = run_file("undef", b"nope\n");
    assert_eq!(lines(&out.stderr)[1], "line 1: 'nope (Undefined variable)");
}

#[test]
fn repl_error_lines_are_exact() {
    let home = std::env::temp_dir().join(format!("oxidedb_review_home_{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    let mut child = bin()
        .env("HOME", &home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"1b+1\nnope\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let _ = std::fs::remove_dir_all(&home);
    assert_eq!(lines(&out.stderr), ["'type", "'nope (Undefined variable)"]);
}

#[test]
fn missing_file_names_the_path() {
    let out = bin().arg("definitely_missing_file.o").output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let e = lines(&out.stderr);
    // line 0 is the banner, which names the file anyway
    assert!(
        e[1..]
            .iter()
            .any(|l| l.contains("definitely_missing_file.o")),
        "{e:?}"
    );
}

#[test]
fn run_file_error_keeps_qerror_as_source() {
    let path = std::env::temp_dir().join(format!("oxidedb_review_src_{}.o", std::process::id()));
    std::fs::write(&path, "1b+1\n").unwrap();
    let err = oxidedb::repl::Repl::new()
        .run_file(path.to_str().unwrap())
        .unwrap_err();
    let _ = std::fs::remove_file(&path);
    assert_eq!(err.downcast_ref::<QError>(), Some(&QError::Type));
    assert_eq!(format!("{err:#}"), "line 1: 'type");
}

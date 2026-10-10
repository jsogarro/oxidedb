use oxidedb::language::lexer::Token;
use oxidedb::{Atom, Interpreter, Lexer, Parser};

fn lex(src: &str) -> Vec<Token> {
    Lexer::new(src).tokenize().unwrap()
}

fn eval(src: &str) -> Atom {
    let ast = Parser::new(lex(src)).parse().unwrap();
    Interpreter::new().evaluate(ast).unwrap()
}

fn parse_err(src: &str) -> String {
    Parser::new(lex(src)).parse().unwrap_err().to_string()
}

#[test]
fn trailing_line_comment() {
    assert_eq!(eval("1 + 2 // note"), Atom::Integer(3));
    assert_eq!(eval("1 + 2 / note"), Atom::Integer(3));
    assert_eq!(eval("1 + 2 /note"), Atom::Integer(3));
    assert_eq!(eval("1 + 2 /"), Atom::Integer(3));
}

#[test]
fn comment_runs_to_end_of_line_only() {
    assert_eq!(
        lex("1 / skip + 2\n3"),
        vec![Token::Integer(1), Token::Integer(3), Token::Eof]
    );
    assert_eq!(lex("1 // a / b"), vec![Token::Integer(1), Token::Eof]);
}

#[test]
fn comment_leading_dot_float() {
    assert_eq!(eval(".5"), Atom::Float(0.5));
    assert_eq!(eval("-.5"), Atom::Float(-0.5));
    assert_eq!(lex("-.5"), vec![Token::Float(-0.5), Token::Eof]);
    assert_eq!(eval("1 + .5 / half"), Atom::Float(1.5));
    // after a noun, `-` glued to `.5` is subtraction unless spaced
    assert_eq!(
        lex("2 -.5"),
        vec![Token::Integer(2), Token::Float(-0.5), Token::Eof]
    );
    assert_eq!(
        lex("2-.5"),
        vec![
            Token::Integer(2),
            Token::Minus,
            Token::Float(0.5),
            Token::Eof
        ]
    );
    assert!(Lexer::new(".").tokenize().is_err());
    assert!(Lexer::new("-.").tokenize().is_err());
}

#[test]
fn comment_whole_line() {
    assert_eq!(lex("/ hi"), vec![Token::Eof]);
    assert_eq!(lex("// hi"), vec![Token::Eof]);
    assert_eq!(lex("/"), vec![Token::Eof]);
    assert_eq!(lex("  / hi"), vec![Token::Eof]);
    assert_eq!(lex("1\t/ c"), vec![Token::Integer(1), Token::Eof]);
}

#[test]
fn comment_glued_slash_is_not_comment() {
    assert_eq!(
        lex("+/1 2"),
        vec![
            Token::Plus,
            Token::Over,
            Token::Integer(1),
            Token::Integer(2),
            Token::Eof
        ]
    );
    assert_eq!(
        lex("+\\1"),
        vec![Token::Plus, Token::Scan, Token::Integer(1), Token::Eof]
    );
    assert_eq!(
        lex("1/2"),
        vec![
            Token::Integer(1),
            Token::Over,
            Token::Integer(2),
            Token::Eof
        ]
    );
}

#[test]
fn glued_adverbs_are_not_yet_implemented() {
    for src in ["1/2", "1\\2", "1+/2", "1+\\2", "(1)/"] {
        let msg = parse_err(src);
        assert!(msg.starts_with("'nyi: adverb"), "{src}: {msg}");
    }
}

#[test]
fn run_file_skips_comment_only_lines() {
    let dir = std::env::temp_dir().join(format!("oxidedb-comments-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let ok = dir.join("ok.o");
    std::fs::write(&ok, "/ hi\n// hi\n  / indented\n1 + 2 / note\n.5\n").unwrap();
    assert!(oxidedb::repl::Repl::new()
        .run_file(ok.to_str().unwrap())
        .is_ok());
    let bad = dir.join("bad.o");
    std::fs::write(&bad, "1/2\n").unwrap();
    assert!(oxidedb::repl::Repl::new()
        .run_file(bad.to_str().unwrap())
        .is_err());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn repl_backslash_exit_still_works() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let home = std::env::temp_dir().join(format!("oxidedb-home-{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_oxidedb"))
        .env("HOME", &home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"/ hi\n1 + 2 // n\n\\\\\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    std::fs::remove_dir_all(&home).ok();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Goodbye!"), "{text}");
    assert!(!text.contains("Error"), "{text}");
    assert!(text.contains('3'), "{text}");
}

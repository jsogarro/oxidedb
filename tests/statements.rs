//! `;` between statements at the top level of a line: left to right, the line's value is the last
//! statement's, a trailing `;` prints nothing. Every expected value was checked against q 4.1.

use oxidedb::repl::Repl;
use oxidedb::{Atom, Interpreter, QError, Value};
use std::io::Write;
use std::process::{Command, Stdio};

fn long(n: i64) -> Option<Value> {
    Some(Value::Atom(Atom::Integer(n)))
}

fn line(i: &mut Interpreter, src: &str) -> Result<Option<Value>, QError> {
    i.eval_line(src)
}

#[test]
fn stmt_value_is_the_last_statement() {
    let mut i = Interpreter::new();
    assert_eq!(line(&mut i, "x:5;x-1"), Ok(long(4)));
    assert_eq!(line(&mut i, "x"), Ok(long(5)));
    assert_eq!(line(&mut i, "a:1;b:2;a+b"), Ok(long(3)));
    assert_eq!(line(&mut i, "1;2;3"), Ok(long(3)));
    assert_eq!(line(&mut i, "x:1;y:x+1;y"), Ok(long(2)));
}

#[test]
fn stmt_run_left_to_right_each_right_to_left() {
    let mut i = Interpreter::new();
    // left to right between statements: the second reads what the first wrote
    assert_eq!(line(&mut i, "x:1;x:x+1;x:x*10;x"), Ok(long(20)));
    // right to left inside one statement
    assert_eq!(line(&mut i, "y:3;y:y+1;y*2"), Ok(long(8)));
    assert_eq!(line(&mut i, "(a:1;a)"), Err(QError::Undefined("a".into())));
}

#[test]
fn stmt_trailing_semicolon_prints_nothing() {
    let mut i = Interpreter::new();
    assert_eq!(line(&mut i, "x:5;"), Ok(None));
    assert_eq!(line(&mut i, "x"), Ok(long(5))); // but the assignment happened
    assert_eq!(line(&mut i, "1;2;"), Ok(None));
    assert_eq!(line(&mut i, "x+1;"), Ok(None));
    assert_eq!(line(&mut i, "1;;"), Ok(None));
    assert_eq!(line(&mut i, ";"), Ok(None));
    assert_eq!(line(&mut i, ";;"), Ok(None));
}

#[test]
fn stmt_empty_statements_are_skipped() {
    let mut i = Interpreter::new();
    assert_eq!(line(&mut i, "1;;2"), Ok(long(2)));
    assert_eq!(line(&mut i, ";1"), Ok(long(1)));
    assert_eq!(line(&mut i, ";;1"), Ok(long(1)));
    assert_eq!(line(&mut i, "x:5;;x"), Ok(long(5)));
}

#[test]
fn stmt_error_stops_the_line_and_earlier_effects_stay() {
    let mut i = Interpreter::new();
    assert_eq!(line(&mut i, "x:0"), Ok(long(0)));
    assert_eq!(
        line(&mut i, "x:1;nope;x:2"),
        Err(QError::Undefined("nope".into()))
    );
    assert_eq!(line(&mut i, "x"), Ok(long(1))); // q: x is 1, the third statement never ran
    assert_eq!(line(&mut i, "y:7;1+`a;y:8"), Err(QError::Type));
    assert_eq!(line(&mut i, "y"), Ok(long(7)));
    // a parse error anywhere runs nothing at all
    assert!(matches!(line(&mut i, "z:1;(2"), Err(QError::Parse(_))));
    assert_eq!(line(&mut i, "z"), Err(QError::Undefined("z".into())));
}

#[test]
fn stmt_semicolons_inside_brackets_parens_and_strings_are_untouched() {
    let mut i = Interpreter::new();
    assert_eq!(line(&mut i, "(1;2);3"), Ok(long(3)));
    assert_eq!(line(&mut i, "(1;2)").unwrap().unwrap().to_string(), "1 2");
    assert_eq!(
        line(&mut i, "(1;`a);(2;`b)").unwrap().unwrap().to_string(),
        "2\n`b"
    );
    assert_eq!(line(&mut i, "\"a;b\";1").unwrap().unwrap().to_string(), "1");
    assert_eq!(
        line(&mut i, "\"a;b\"").unwrap().unwrap().to_string(),
        "\"a;b\""
    );
    // a list item is still not a statement: `(1;)` is an elided item, not "1 then nothing"
    assert_eq!(
        line(&mut i, "(1;)"),
        Err(QError::Nyi("elided list item".into()))
    );
    assert_eq!(line(&mut i, "til[2;3]"), Err(QError::Rank));
    assert_eq!(line(&mut i, "count[(1;2)];"), Ok(None));
}

#[test]
fn stmt_comments_and_item_assignment() {
    let mut i = Interpreter::new();
    assert_eq!(line(&mut i, "x:1; / a comment"), Ok(None));
    assert_eq!(line(&mut i, "x:2;x+1 / a comment"), Ok(long(3)));
    // v[0]:1; v works: the assignment value ends at the `;`
    assert_eq!(
        line(&mut i, "v:10 20 30"),
        Ok(Some(i.get("v").unwrap().clone()))
    );
    assert_eq!(
        line(&mut i, "v[0]:1;v").unwrap().unwrap().to_string(),
        "1 20 30"
    );
    assert_eq!(line(&mut i, "v[1 2]:7 8;").unwrap(), None);
    assert_eq!(i.get("v").unwrap().to_string(), "1 7 8");
    // an assignment's value stops at the semicolon, so it does not swallow the next statement
    assert_eq!(line(&mut i, "w:v[0]+1;w*2"), Ok(long(4)));
    assert_eq!(line(&mut i, "a:b:3;a+b"), Ok(long(6)));
}

#[test]
fn stmt_the_repl_prints_nothing_for_a_trailing_semicolon() {
    let mut repl = Repl::new();
    assert_eq!(repl.eval_line("x:5;x-1").unwrap(), Some("4".to_string()));
    assert_eq!(repl.eval_line("x:5;").unwrap(), None);
    assert_eq!(repl.eval_line("x;").unwrap(), None);
    assert_eq!(repl.eval_line(";").unwrap(), None);
    // plain assignment keeps echoing (pending decision), only the trailing `;` silences
    assert_eq!(repl.eval_line("x:6").unwrap(), Some("6".to_string()));
}

const BIN: &str = env!("CARGO_BIN_EXE_oxidedb");

#[test]
fn stmt_file_mode() {
    let path = std::env::temp_dir().join(format!("stmt_{}.o", std::process::id()));
    std::fs::write(&path, "x:5;x-1\ny:2;\n;\ny;y+x / c\nz:1;;z\n").unwrap();
    let out = Command::new(BIN).arg(&path).output().unwrap();
    let _ = std::fs::remove_file(&path);
    assert!(out.status.success(), "{:?}", out.status);
    // line 1: 4; line 2: nothing; line 3: nothing; line 4: 7; line 5: 1
    assert_eq!(String::from_utf8_lossy(&out.stdout), "4\n7\n1\n");
}

#[test]
fn stmt_file_mode_error_stops_at_the_line() {
    let path = std::env::temp_dir().join(format!("stmt_err_{}.o", std::process::id()));
    std::fs::write(&path, "x:1;nope;x:2\n3\n").unwrap();
    let out = Command::new(BIN).arg(&path).output().unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("line 1"));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
}

#[test]
fn stmt_stdin_mode() {
    let mut child = Command::new(BIN)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    write!(
        child.stdin.take().unwrap(),
        "x:5;x-1\nx:7;\nx\nx:1;nope;x:2\nx\nexit\n"
    )
    .unwrap();
    let out = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let shown: Vec<&str> = stdout
        .lines()
        .filter(|l| {
            !l.is_empty() && !l.starts_with("OxideDB") && !l.starts_with("Type") && *l != "Goodbye!"
        })
        .collect();
    assert_eq!(shown, ["4", "7", "1"], "{stdout}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("'nope"));
}

#[test]
fn stmt_a_plain_line_parses_to_itself() {
    use oxidedb::language::ast::Expr;
    use oxidedb::{Lexer, Parser};
    let parse = |src: &str| Parser::new(Lexer::new(src).tokenize().unwrap()).parse();
    assert!(matches!(parse("1+2"), Ok(Expr::BinaryOp { .. })));
    assert!(matches!(
        parse("1+2;"),
        Ok(Expr::Sequence { silent: true, .. })
    ));
    assert!(matches!(
        parse("1;2"),
        Ok(Expr::Sequence { silent: false, ref statements }) if statements.len() == 2
    ));
}

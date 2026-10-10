use oxidedb::language::interpreter::Interpreter;
use oxidedb::repl::Repl;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn run(name: &str, bytes: &[u8]) -> Output {
    let path: PathBuf = std::env::temp_dir().join(format!(
        "oxidedb_run_file_{}_{}.o",
        std::process::id(),
        name
    ));
    std::fs::write(&path, bytes).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_oxidedb"))
        .arg(&path)
        .output()
        .unwrap();
    let _ = std::fs::remove_file(&path);
    out
}

#[test]
fn run_file_accepts_utf8_bom() {
    let out = run("bom", b"\xEF\xBB\xBF1+2\n");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "3");
}

#[test]
fn run_file_reports_error_once_and_fails() {
    let out = run("err", b"1+2\n1 +\n9\n");
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(stderr.matches("'parse").count(), 1, "stderr: {stderr}");
    assert!(
        stderr.contains("'parse: unexpected end of input (line 2)"),
        "stderr: {stderr}"
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "3");
}

#[test]
fn run_file_crlf_ok() {
    let out = run("crlf", b"1+2\r\n4*5\r\n");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "3\n20");
}

#[test]
fn repl_eval_line_ok_and_err() {
    let mut repl = Repl::new();
    assert_eq!(repl.eval_line("1+2").unwrap(), Some("3".to_string()));
    assert!(repl.eval_line("1 +").is_err());
    assert_eq!(repl.eval_line("2*3").unwrap(), Some("6".to_string()));
    assert_eq!(repl.eval_line("// note").unwrap(), None);
}

#[test]
fn interpreter_eval_line_some_none_err() {
    let mut i = Interpreter::new();
    assert!(i.eval_line("1+2").unwrap().is_some());
    assert!(i.eval_line("// only a comment").unwrap().is_none());
    assert!(i.eval_line("   ").unwrap().is_none());
    assert!(i.eval_line("1 +").is_err());
    assert!(i.eval_line("undefined_var").is_err()); // evaluation-stage error
}

#[test]
fn repl_errors_go_to_stderr_results_to_stdout() {
    let home = std::env::temp_dir().join(format!("oxidedb_home_{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_oxidedb"))
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
        .write_all(b"1 +\n/ note\n2+3\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let (o, e) = (
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(e.contains("'parse"), "stderr: {e}");
    assert!(!o.contains("'parse"), "stdout: {o}");
    assert!(o.contains('5'), "stdout: {o}");
    // Two banner lines, then only the result: the comment-only line prints nothing.
    let shown = o.replace("oxidedb> ", "");
    let lines: Vec<&str> = shown
        .lines()
        .filter(|l| !l.is_empty() && *l != "Goodbye!")
        .collect();
    assert_eq!(lines.len(), 3, "stdout: {o}");
    assert_eq!(lines[2], "5", "stdout: {o}");
    assert!(!home.join(".oxidedb_history").exists());
    let _ = std::fs::remove_dir_all(&home);
}

use oxidedb::repl::Repl;
use std::path::PathBuf;
use std::process::{Command, Output};

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
    assert_eq!(stderr.matches("Error").count(), 1, "stderr: {stderr}");
    assert!(stderr.contains("line 2"), "stderr: {stderr}");
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
    assert_eq!(repl.eval_line("1+2").unwrap(), "3");
    assert!(repl.eval_line("1 +").is_err());
    assert_eq!(repl.eval_line("2*3").unwrap(), "6");
}

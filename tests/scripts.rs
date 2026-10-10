//! Script-file rules shared with q: `\\` ends the script, `/` alone opens a block comment that a
//! `\` alone closes (blocks nest), and a `\` alone outside a block ends the script too.
use oxidedb::repl::Repl;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn run(name: &str, src: &str) -> Output {
    let path: PathBuf =
        std::env::temp_dir().join(format!("oxidedb_scripts_{}_{}.o", std::process::id(), name));
    std::fs::write(&path, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_oxidedb"))
        .arg(&path)
        .output()
        .unwrap();
    let _ = std::fs::remove_file(&path);
    out
}

/// Runs a script that must succeed and returns its stdout.
fn ok(name: &str, src: &str) -> String {
    let out = run(name, src);
    assert!(
        out.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn script_double_backslash_ends_script() {
    assert_eq!(ok("dbs", "1\n\\\\\n2\n"), "1\n");
}

#[test]
fn script_double_backslash_ignores_everything_after() {
    assert_eq!(ok("dbs_junk", "1\n\\\\\n\"unterminated\n(((\n"), "1\n");
}

#[test]
fn script_double_backslash_tolerates_trailing_whitespace_and_crlf() {
    assert_eq!(ok("dbs_ws", "1\n\\\\ \t\n2\n"), "1\n");
    assert_eq!(ok("dbs_crlf", "1\r\n\\\\\r\n2\r\n"), "1\n");
}

#[test]
fn script_double_backslash_as_first_line() {
    assert_eq!(ok("dbs_first", "\\\\\n1\n"), "");
}

#[test]
fn script_double_backslash_inside_block_does_not_end_script() {
    assert_eq!(ok("dbs_block", "/\n\\\\\n\\\n1\n"), "1\n");
}

#[test]
fn script_double_backslash_followed_by_text_ends_script() {
    // q ends the script at `\\` followed by whitespace and anything
    assert_eq!(ok("dbs_text", "1\n\\\\ goodbye\n2\n"), "1\n");
    assert_eq!(ok("dbs_tab", "1\n\\\\\tx\n2\n"), "1\n");
}

#[test]
fn script_double_backslash_glued_to_text_is_a_system_command_error() {
    let out = run("dbs_glued", "1\n\\\\ls\n2\n");
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("line 2: 'nyi: system command"),
        "stderr: {stderr}"
    );
    assert!(!stderr.contains("adverb"), "stderr: {stderr}");
}

#[test]
fn repl_double_backslash_with_text_quits_and_glued_text_errors() {
    let o = repl("1\n\\\\ bye\n2\n");
    assert!(o.contains("Goodbye!") && !o.contains('2'), "stdout: {o}");
    let mut child = Command::new(env!("CARGO_BIN_EXE_oxidedb"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"\\\\ls\n").unwrap();
    let out = child.wait_with_output().unwrap();
    let e = String::from_utf8_lossy(&out.stderr);
    assert!(e.contains("'nyi: system command"), "stderr: {e}");
}

#[test]
fn script_indented_double_backslash_is_not_the_exit_command() {
    let out = run("dbs_indent", "1\n \\\\\n2\n");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}

#[test]
fn script_block_comment_skipped() {
    assert_eq!(ok("blk", "1\n/\nx:5\n2+2\n\\\n3\n"), "1\n3\n");
    // the skipped body is not lexed: any text goes
    assert_eq!(ok("blk_junk", "/\n\"open\n(((  `\\x\n\\\n7\n"), "7\n");
    // the body was never run
    let out = run("blk_unrun", "/\nx:5\n\\\nx\n");
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("line 4:"));
}

#[test]
fn script_block_comment_unterminated_runs_to_end_of_file() {
    assert_eq!(ok("blk_open", "1\n/\n2\n3\n"), "1\n");
}

#[test]
fn script_block_comment_markers_tolerate_trailing_whitespace_and_crlf() {
    assert_eq!(ok("blk_ws", "1\n/ \t\n2\n\\  \n3\n"), "1\n3\n");
    assert_eq!(ok("blk_crlf", "1\r\n/\r\n2\r\n\\\r\n3\r\n"), "1\n3\n");
}

#[test]
fn script_block_comments_nest() {
    // `/` inside a block opens a second level: the first `\` only closes it
    assert_eq!(ok("blk_nest", "1\n/\n2\n/\n3\n\\\n4\n\\\n5\n"), "1\n5\n");
}

#[test]
fn script_slash_with_text_or_indent_is_an_ordinary_comment() {
    assert_eq!(ok("blk_text", "/ note\n1\n/note\n2\n"), "1\n2\n");
    assert_eq!(ok("blk_indent", " /\n1\n"), "1\n");
    assert_eq!(ok("blk_tab", "\t/\n1\n"), "1\n");
}

#[test]
fn script_lone_backslash_ends_script() {
    assert_eq!(ok("lone", "1\n\\\n2\n"), "1\n");
    assert_eq!(ok("lone_junk", "1\n\\\n\"unterminated\n(((\n"), "1\n");
    assert_eq!(ok("lone_ws", "1\n\\ \t\n2\n"), "1\n");
}

#[test]
fn script_lone_backslash_after_a_block_ends_the_script() {
    // the first `\` closes the block, the second has no block to close
    assert_eq!(ok("lone_after", "/\n0\n\\\n1\n\\\n2\n"), "1\n");
}

#[test]
fn script_indented_lone_backslash_is_not_a_terminator() {
    let out = run("lone_indent", "1\n \\\n2\n");
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn script_error_after_block_comment_reports_file_line() {
    let out = run("blk_line", "1\n/\nskipped\nskipped\n\\\n2\n1 +\n");
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("line 7: 'parse"), "stderr: {stderr}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n2\n");
}

#[test]
fn missing_script_error_names_the_path() {
    let out = Command::new(env!("CARGO_BIN_EXE_oxidedb"))
        .arg("no_such_dir/missing.o")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("cannot read no_such_dir/missing.o"),
        "stderr: {stderr}"
    );
}

/// The interactive loop: `\\` quits, and `/` alone is a plain comment line (no block mode).
fn repl(input: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_oxidedb"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&out.stdout).replace("oxidedb> ", "")
}

#[test]
fn repl_double_backslash_quits() {
    let o = repl("1\n\\\\\n2\n");
    assert!(o.contains("Goodbye!"), "stdout: {o}");
    assert!(!o.contains('2'), "stdout: {o}");
}

#[test]
fn repl_lone_slash_is_not_a_block_comment() {
    let o = repl("/\n7\n");
    assert!(o.lines().any(|l| l == "7"), "stdout: {o}");
}

#[test]
fn history_path_prefers_home_then_userprofile() {
    let p = |h: Option<&str>, u: Option<&str>| {
        Repl::history_path_from(h.map(Into::into), u.map(Into::into))
    };
    assert_eq!(
        p(Some("/h"), Some("/u")),
        Some("/h/.oxidedb_history".into())
    );
    assert_eq!(p(None, Some("/u")), Some("/u/.oxidedb_history".into()));
    assert_eq!(p(None, None), None);
    // set-but-empty counts as unset
    assert_eq!(p(Some(""), Some("/u")), Some("/u/.oxidedb_history".into()));
    assert_eq!(p(Some(""), Some("")), None);
    assert_eq!(p(None, Some("")), None);
}

//! Each input is parsed in a child process (this test binary, re-executed) so
//! that a stack-overflow abort or a panic is reported as a test failure
//! instead of killing the harness, and a hang is cut off by a timeout.

use oxidedb::{Lexer, Parser};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SRC_VAR: &str = "PARSE_STRICT_SRC";

/// Child entry point: a no-op unless `SRC_VAR` is set.
#[test]
fn parse_strict_probe() {
    let Ok(src) = std::env::var(SRC_VAR) else {
        return;
    };
    let parsed = Lexer::new(&src)
        .tokenize()
        .and_then(|tokens| Parser::new(tokens).parse());
    println!("RESULT:{}", if parsed.is_ok() { "OK" } else { "ERR" });
}

/// Asserts that parsing `src` returns `Err` cleanly within 5 seconds.
fn assert_parse_err(src: &str) {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "parse_strict_probe",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(SRC_VAR, src)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() > deadline {
            child.kill().unwrap();
            panic!("parsing {src:?} did not finish within 5s");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let out = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("RESULT:ERR"),
        "parsing {src:?} should return Err; status {:?}, stdout {stdout:?}, stderr {:?}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn parse_strict_trailing_tokens_rejected() {
    for src in ["1 2 )", "3 )", "1;2 )", "1;(2", "1:2", "2x"] {
        assert_parse_err(src);
    }
}

#[test]
fn parse_strict_unbalanced_paren_errors() {
    for src in ["(1+2", "((", "(", "1+("] {
        assert_parse_err(src);
    }
}

#[test]
fn parse_strict_empty_input_errors() {
    for src in ["", "   "] {
        assert_parse_err(src);
    }
}

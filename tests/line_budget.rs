//! One budget per parsed line bounds the AST depth (nesting and chain lengths used to
//! multiply into a stack overflow), and the binary runs the interpreter on its own stack.
//! Library shapes run in a child process so a regression fails the test, not the harness.

use oxidedb::{Interpreter, Lexer, Parser, Value};
use std::io::Write;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

const STACK_VAR: &str = "LINE_BUDGET_STACK_KB";

/// Child entry point: a no-op unless `STACK_VAR` is set. Reads the line from stdin, then lexes,
/// parses and evaluates it on a thread with that many KB of stack.
#[test]
fn line_budget_probe() {
    let Ok(kb) = std::env::var(STACK_VAR) else {
        return;
    };
    let kb: usize = kb.parse().unwrap();
    let mut src = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut src).unwrap();
    let line = std::thread::Builder::new()
        .stack_size(kb * 1024)
        .spawn(move || {
            let tokens = match Lexer::new(&src).tokenize() {
                Ok(t) => t,
                Err(e) => return format!("ERR:{e}"),
            };
            let ast = match Parser::new(tokens).parse() {
                Ok(a) => a,
                Err(e) => return format!("ERR:{e}"),
            };
            match Interpreter::new().evaluate(ast) {
                Ok(Value::Atom(a)) => format!("OK:{a}"),
                Ok(v) => format!("OK:{v}"),
                Err(e) => format!("ERR:{e}"),
            }
        })
        .unwrap()
        .join()
        .unwrap();
    println!("RESULT:{line}");
}

/// Waits for `child` at most 60 s, killing it (and failing the test) on a hang.
fn wait_timeout(mut child: Child, what: &str) -> Output {
    let deadline = Instant::now() + Duration::from_secs(60);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() > deadline {
            child.kill().unwrap();
            panic!("{what} did not finish within 60s");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output().unwrap()
}

/// Runs the probe in a child; `Ok(value text)`, `Err(error text)`, or panics on an abort.
fn run_child(src: &str, stack_kb: usize) -> Result<String, String> {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "line_budget_probe", "--nocapture"])
        .env(STACK_VAR, stack_kb.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // The child reads all of stdin; feed it from a thread so a full pipe cannot deadlock.
    let mut stdin = child.stdin.take().unwrap();
    let input = src.to_string();
    let feeder = std::thread::spawn(move || {
        let _ = stdin.write_all(input.as_bytes());
    });
    let out = wait_timeout(child, "probe");
    feeder.join().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let result = stdout
        .lines()
        .find_map(|l| l.strip_prefix("RESULT:"))
        .unwrap_or_else(|| {
            panic!(
                "child aborted ({:?}) on a {}-byte line: {}",
                out.status,
                src.len(),
                String::from_utf8_lossy(&out.stderr)
                    .chars()
                    .take(200)
                    .collect::<String>()
            )
        });
    match result.split_once(':').unwrap() {
        ("OK", v) => Ok(v.to_string()),
        (_, e) => Err(e.to_string()),
    }
}

fn assert_rejected(src: &str) {
    let e = run_child(src, 8192).expect_err("should be rejected");
    assert!(
        e.contains("too long") || e.contains("nested too deeply"),
        "unexpected error {e}"
    );
}

/// `levels` parentheses, each opened after a `terms`-term `1+` chain.
fn nested(levels: usize, terms: usize) -> String {
    format!(
        "{}1{}",
        format!("{}(", "1+".repeat(terms)).repeat(levels),
        ")".repeat(levels)
    )
}

#[test]
fn nesting_times_chain_is_rejected() {
    assert_rejected(&nested(127, 217)); // the 55 KB reproducer
    assert_rejected(&nested(32, 858));
    assert_rejected(&nested(2, 1762)); // the debug-build reproducer
}

#[test]
fn every_verb_spends_from_the_budget() {
    for verb in [
        "-", "*", "%", "=", "<", ">", "<>", "<=", ">=", "#", ",", "!",
    ] {
        let level = format!("{}(", format!("1{verb}").repeat(217));
        assert_rejected(&format!("{}1{}", level.repeat(127), ")".repeat(127)));
        let flat = format!("{}1", format!("1{verb}").repeat(2000));
        assert!(run_child(&flat, 8192).unwrap_err().contains("too long"));
    }
    // Under the budget a non-arithmetic chain still evaluates.
    assert!(run_child(&format!("{}1", "1,".repeat(1999)), 8192).is_ok());
}

/// Not rejected: the line parses, and evaluating `1 2 3[0][0]...` is a plain `'type`.
fn assert_parses(src: &str) {
    assert_eq!(run_child(src, 8192), Err("'type".to_string()), "{src:.60}");
}

#[test]
fn application_and_bracket_links_spend_from_the_budget() {
    // juxtaposition: flat boundary (the line spends one, each join one) ...
    assert_eq!(
        run_child(&format!("{}1", "count ".repeat(1999)), 8192).unwrap(),
        "1"
    );
    assert!(run_child(&format!("{}1", "count ".repeat(2000)), 8192)
        .unwrap_err()
        .contains("too long"));
    // ... and nested 127 levels deep, each opened after a long chain
    let level = format!("{}(", "count ".repeat(217));
    assert_rejected(&format!("{}1{}", level.repeat(127), ")".repeat(127)));
    // brackets: a suffix spends a link plus its argument expression, so 1 + 2k <= 2000
    assert_parses(&format!("1 2 3{}", "[0]".repeat(999)));
    assert!(run_child(&format!("1 2 3{}", "[0]".repeat(1000)), 8192)
        .unwrap_err()
        .contains("too long"));
    // the abort shapes: chains and suffix links nested 127 levels deep
    assert_rejected(&format!(
        "{}1{}",
        ("count ".repeat(147) + "(").repeat(127),
        ")".repeat(127)
    ));
    assert_rejected(&format!(
        "{}0{}",
        ("1+".repeat(146) + "v[").repeat(127),
        "]".repeat(127)
    ));
    assert_rejected(&format!(
        "{}v{}",
        "(".repeat(127),
        (")".to_string() + &"[0 1 2]".repeat(147)).repeat(127)
    ));
    // `v[ v[ ... ][0][0]... ][0][0]...`: suffix links hung off the first argument of every level
    let spine = |k: usize| {
        format!(
            "{}0{}",
            "v[".repeat(127),
            ("]".to_string() + &"[0]".repeat(k)).repeat(127)
        )
    };
    assert_rejected(&spine(146));
    assert_rejected(&spine(8));
    // under the budget the same shape still parses (127 * (2 + 2k) + 1 <= 2000 for k = 6)
    assert_parses(&spine(6).replace("v[", "1 2 3["));
    // bracket arguments go through `expression()`: nesting is capped at 128 levels
    assert!(run_child(
        &format!("{}0{}", "1 2 3[".repeat(128), "]".repeat(128)),
        8192
    )
    .unwrap_err()
    .contains("nested too deeply"));
}

#[test]
fn leading_minus_variant_is_rejected() {
    for (levels, terms) in [(127, 217), (32, 858)] {
        assert_rejected(&format!("{}1", ("1+".repeat(terms) + "- ").repeat(levels)));
    }
}

#[test]
fn assignment_spends_from_the_budget() {
    assert_rejected(&format!(
        "{}1",
        ("a:".to_string() + &"1+".repeat(217)).repeat(127)
    ));
    let level = "a:".to_string() + &"1+".repeat(217) + "(";
    assert_rejected(&format!("{}1{}", level.repeat(127), ")".repeat(127)));
    let colons = |ops: usize| format!("{}{}1", "a:".repeat(127), "1+".repeat(ops));
    assert!(run_child(&colons(1872), 8192).is_ok());
    assert!(run_child(&colons(1873), 8192)
        .unwrap_err()
        .contains("too long"));
}

#[test]
fn mixed_shapes_are_rejected() {
    // Parentheses, minus and an assignment-free chain interleaved.
    let level = "1+".repeat(300) + "(- " + &"1+".repeat(300) + "(";
    assert_rejected(&format!("{}1{}", level.repeat(20), ")".repeat(40)));
}

#[test]
fn legitimate_large_inputs_evaluate() {
    assert_eq!(
        run_child(&format!("{}1", "1+".repeat(1999)), 8192).unwrap(),
        "2000"
    );
    let parens = format!("{}1{}", "(".repeat(127), ")".repeat(127));
    assert_eq!(run_child(&parens, 8192).unwrap(), "1");
    let siblings = format!("{}1", "(1+1+1+1+1+1+1+1+1+1+1)+".repeat(100));
    assert_eq!(run_child(&siblings, 8192).unwrap(), "1101");
}

#[test]
fn budget_boundary_is_pinned() {
    // One budget: a chain of N operators costs N plus one for the line itself.
    let chain = |ops: usize| format!("{}1", "1+".repeat(ops));
    assert!(run_child(&chain(1999), 8192).is_ok());
    assert!(run_child(&chain(2000), 8192)
        .unwrap_err()
        .contains("too long"));
    // The same budget is spent across levels: 1 (line) + 1 (paren) + 1998 ops fits, 1999 does not.
    let split = |a: usize, b: usize| format!("{}({}1)", "1+".repeat(a), "1+".repeat(b));
    assert!(run_child(&split(1000, 998), 8192).is_ok());
    assert!(run_child(&split(1000, 999), 8192)
        .unwrap_err()
        .contains("too long"));
    // Siblings spend from the same budget: a level that returns does not refund it.
    let sib = |a: usize, b: usize, lead: &str| {
        format!("({lead}{}1)+({lead}{}1)+1", "1+".repeat(a), "1+".repeat(b))
    };
    assert!(run_child(&sib(997, 998, ""), 8192).is_ok()); // 1 + 2 parens + 2 joins + 1995
    assert!(run_child(&sib(998, 998, ""), 8192)
        .unwrap_err()
        .contains("too long"));
    // ... and a minus inside the parentheses spends one more each.
    assert!(run_child(&sib(996, 997, "- "), 8192).is_ok());
    assert!(run_child(&sib(997, 997, "- "), 8192)
        .unwrap_err()
        .contains("too long"));
    // ... and so does an assignment value, in either position (`(a:..)` and `(1+a:..)`).
    assert!(run_child(&sib(996, 997, "a:"), 8192).is_ok());
    assert!(run_child(&sib(997, 997, "a:"), 8192)
        .unwrap_err()
        .contains("too long"));
    assert!(run_child(&sib(995, 996, "1+a:"), 8192).is_ok());
    assert!(run_child(&sib(996, 996, "1+a:"), 8192)
        .unwrap_err()
        .contains("too long"));
    // Siblings made of bracket suffixes share the budget as well: 1 + 2a + 1 join + 2b <= 2000.
    let suffixes =
        |a: usize, b: usize| format!("1 2 3{}+1 2 3{}", "[0]".repeat(a), "[0]".repeat(b));
    assert_parses(&suffixes(500, 499));
    assert!(run_child(&suffixes(500, 500), 8192)
        .unwrap_err()
        .contains("too long"));
    // Unary minus spends too: each `- ` is a nested expression.
    let minus = |n: usize| format!("{}1", "- ".repeat(n));
    assert!(run_child(&format!("{}{}", "1+".repeat(1872), minus(127)), 8192).is_ok());
    assert!(
        run_child(&format!("{}{}", "1+".repeat(1873), minus(127)), 8192)
            .unwrap_err()
            .contains("too long")
    );
}

// ---- the binary ----

const BIN: &str = env!("CARGO_BIN_EXE_oxidedb");

fn reproducer() -> String {
    nested(127, 217)
}

fn assert_clean_parse_error(out: &Output) {
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("too long") || err.contains("nested too deeply"),
        "stderr: {}",
        err.chars().take(300).collect::<String>()
    );
    assert!(!err.contains("stack overflow"), "aborted: {err}");
    assert!(!err.contains("SIGABRT"));
}

#[test]
fn binary_file_mode_rejects_reproducer() {
    let path = std::env::temp_dir().join(format!("line_budget_{}.o", std::process::id()));
    std::fs::write(&path, reproducer() + "\n").unwrap();
    let out = Command::new(BIN).arg(&path).output().unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(out.status.code(), Some(1), "{:?}", out.status);
    assert_clean_parse_error(&out);
}

#[test]
fn binary_stdin_mode_rejects_reproducer_and_continues() {
    let mut child = Command::new(BIN)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    // The session survives the rejected line: the next line still evaluates.
    write!(stdin, "x:41\n{}\nx+1\nexit\n", reproducer()).unwrap();
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{:?}", out.status);
    assert_clean_parse_error(&out);
    assert!(String::from_utf8_lossy(&out.stdout).contains("42"));
}

/// The dedicated interpreter stack: with the main thread cut to 512 KB a 1,999-operator chain
/// (about 3 MB in a debug build) still runs, which only the spawned thread makes possible.
#[cfg(unix)]
#[test]
fn binary_runs_on_its_own_stack() {
    let path = std::env::temp_dir().join(format!("line_budget_stack_{}.o", std::process::id()));
    std::fs::write(&path, format!("{}1\n", "1+".repeat(1999))).unwrap();
    let out = Command::new("sh")
        .arg("-c")
        .arg("ulimit -s 512 && exec \"$0\" \"$1\"")
        .arg(BIN)
        .arg(&path)
        .output()
        .unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "2000",
        "status {:?}, stderr {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A write failure on stdout (closed pipe) is a panic in `println!`; it must reach the exit
/// status through the interpreter thread, not be swallowed.
#[test]
fn binary_propagates_a_panic_from_the_interpreter_thread() {
    let mut child = Command::new(BIN)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let mut stdin = child.stdin.take().unwrap();
    let _ = write!(stdin, "1+1\nexit\n");
    drop(stdin);
    let out = wait_timeout(child, "binary");
    assert_eq!(out.status.code(), Some(101), "{:?}", out.status);
}

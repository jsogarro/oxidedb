//! Keeps "O for Humans" honest: every expected output in the book is
//! evaluated with the real interpreter. The checkers take text so that they
//! can be tested against in-memory fixtures (see the `checker_*` tests).

use oxidedb::Interpreter;
use std::fs;
use std::path::{Path, PathBuf};

const PROMPT: &str = "oxidedb> ";
const MARKER: &str = "// Expected output: ";

/// Evaluate one line the way the binary does: comment-only and blank lines
/// print nothing (empty string), errors print as the q-style error text, which
/// always starts with a quote (`'type`); no atom prints that way.
fn run_line(interp: &mut Interpreter, input: &str) -> Option<String> {
    match interp.eval_line(input) {
        Ok(Some(atom)) => Some(format!("{}", atom)),
        Ok(None) => None,
        Err(e) => Some(e.to_string()),
    }
}

/// Check a `.o` file. Returns one message per problem.
fn check_o_file(name: &str, text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let mut interp = Interpreter::new();
    let mut pending: Option<(usize, String)> = None;
    let mut expectations = 0;
    for (i, raw) in text.lines().enumerate() {
        let (n, line) = (i + 1, raw.trim());
        if line.starts_with('/') {
            let lower = line.to_lowercase();
            if let Some(expected) = line.strip_prefix(MARKER).map(str::trim) {
                if let Some((en, _)) = pending {
                    problems.push(format!(
                        "{}:{}: expectation follows the one at line {} before any expression",
                        name, n, en
                    ));
                }
                pending = Some((n, expected.to_string()));
                continue;
            }
            if lower.contains("expected") && (lower.contains("output") || lower.contains(':')) {
                problems.push(format!(
                    "{}:{}: looks like an expectation but is not exactly `{}X`: {}",
                    name, n, MARKER, line
                ));
            }
        }
        let Some(actual) = run_line(&mut interp, line) else {
            continue; // blank or comment-only
        };
        match pending.take() {
            Some((en, expected)) => {
                expectations += 1;
                if actual.starts_with('\'') {
                    problems.push(format!(
                        "{}:{}: `{}` is an error ({}): example files must run to completion; move error samples to the chapter",
                        name, n, line, actual
                    ));
                    continue;
                }
                if actual != expected {
                    problems.push(format!(
                        "{}:{}: `{}` (expectation at line {})\n    expected: {}\n    actual:   {}",
                        name, n, line, en, expected, actual
                    ));
                }
            }
            None if actual.starts_with('\'') => {
                problems.push(format!(
                    "{}:{}: `{}` failed unexpectedly: {}",
                    name, n, line, actual
                ));
            }
            None => {}
        }
    }
    if let Some((en, _)) = pending {
        problems.push(format!(
            "{}:{}: expectation has no expression after it",
            name, en
        ));
    }
    if expectations == 0 {
        problems.push(format!(
            "{}: contains no `{}X` lines",
            name,
            MARKER.trim_end()
        ));
    }
    problems
}

struct Fence {
    indented: bool,
    ch: char,
    len: usize,
    info: String,
}

fn fence_open(line: &str) -> Option<Fence> {
    let t = line.trim_start();
    let ch = t.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let len = t.chars().take_while(|c| *c == ch).count();
    (len >= 3).then(|| Fence {
        indented: t.len() != line.len(),
        ch,
        len,
        info: t[len..].trim().to_string(),
    })
}

fn fence_closes(f: &Fence, line: &str) -> bool {
    let t = line.trim();
    t.len() >= f.len && t.chars().all(|c| c == f.ch)
}

/// Check a chapter. Only plain ``` fences are run; a fence containing
/// `oxidedb>` that is not run must be labelled ```text.
fn check_chapter(name: &str, text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let mut interp = Interpreter::new(); // one fresh session per chapter
    let lines: Vec<&str> = text.lines().collect();
    let mut checked = 0;
    let mut i = 0;
    while i < lines.len() {
        let Some(f) = fence_open(lines[i]) else {
            i += 1;
            continue;
        };
        let start = i + 1;
        let mut end = None;
        for (j, l) in lines.iter().enumerate().skip(start) {
            if fence_closes(&f, l) {
                end = Some(j);
                break;
            }
        }
        let Some(end) = end else {
            problems.push(format!("{}:{}: fence is never closed", name, start));
            break;
        };
        let body = &lines[start..end];
        let runs = !f.indented && f.ch == '`' && f.len == 3 && f.info.is_empty();
        let text_label = !f.indented && f.ch == '`' && f.len == 3 && f.info == "text";
        if runs {
            checked += check_transcript(name, start, body, &mut interp, &mut problems);
        } else if !text_label {
            for (k, l) in body.iter().enumerate() {
                if l.contains("oxidedb>") {
                    problems.push(format!(
                        "{}:{}: `oxidedb>` inside a fence that is not run; use a plain ``` fence, or label it ```text",
                        name,
                        start + k + 1
                    ));
                }
            }
        }
        i = end + 1;
    }
    if checked == 0 {
        problems.push(format!("{}: contains no checked `oxidedb>` samples", name));
    }
    problems
}

/// Run one transcript block; returns the number of prompts checked.
fn check_transcript(
    file: &str,
    first_line: usize, // 0-based index of the first body line
    body: &[&str],
    interp: &mut Interpreter,
    problems: &mut Vec<String>,
) -> usize {
    let mut checked = 0;
    let mut idx = 0;
    while idx < body.len() {
        let (n, line) = (first_line + idx + 1, body[idx]);
        idx += 1;
        if !line.starts_with(PROMPT) {
            if line.contains("oxidedb>") {
                problems.push(format!(
                    "{}:{}: `oxidedb>` not at the start of the line followed by a space: {}",
                    file, n, line
                ));
            } else if !line.trim().is_empty() {
                problems.push(format!(
                    "{}:{}: output with no prompt before it: {}",
                    file, n, line
                ));
            }
            continue;
        }
        let input = line[PROMPT.len()..].trim();
        if input.is_empty() {
            problems.push(format!("{}:{}: empty prompt in a checked block", file, n));
            continue;
        }
        let mut expected = Vec::new();
        while idx < body.len() && !body[idx].contains("oxidedb>") {
            expected.push(body[idx]);
            idx += 1;
        }
        let expected = expected.join("\n");
        let actual = run_line(interp, input).unwrap_or_default();
        checked += 1;
        if actual != expected.trim_end() {
            problems.push(format!(
                "{}:{}: `{}`\n    expected: {}\n    actual:   {}",
                file, n, input, expected, actual
            ));
        }
    }
    checked
}

fn book_files(dir: &str, ext: &str) -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("book").join(dir);
    let mut files: Vec<PathBuf> = fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("cannot read {}: {}", root.display(), e))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == ext))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no .{} files in {}", ext, root.display());
    files
}

fn check_all(dir: &str, ext: &str, check: fn(&str, &str) -> Vec<String>) {
    let mut problems = Vec::new();
    for path in book_files(dir, ext) {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        problems.extend(check(&name, &fs::read_to_string(&path).unwrap()));
    }
    assert!(problems.is_empty(), "\n{}\n", problems.join("\n"));
}

#[test]
fn book_examples_match_expected_output() {
    check_all("examples", "o", check_o_file);
}

#[test]
fn book_chapter_samples_match() {
    check_all("chapters", "md", check_chapter);
}

#[test]
fn book_example_files_run_to_completion() {
    for path in book_files("examples", "o") {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_oxidedb"))
            .arg(&path)
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{} exited with {:?}: {}",
            path.display(),
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn book_readme_links_resolve() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("book");
    let text = fs::read_to_string(dir.join("README.md")).unwrap();
    let mut missing = Vec::new();
    let mut seen = 0;
    for part in text.split("](").skip(1) {
        let target = part.split(')').next().unwrap();
        if target.contains("://") || target.starts_with('#') || target.starts_with("mailto:") {
            continue;
        }
        seen += 1;
        let file = target.split('#').next().unwrap();
        if !dir.join(file).exists() {
            missing.push(target.to_string());
        }
    }
    assert!(seen > 0, "book/README.md has no relative links");
    assert!(
        missing.is_empty(),
        "broken links in book/README.md: {:?}",
        missing
    );
}

// ---- tests of the checkers themselves, on in-memory fixtures ----

fn reports(problems: &[String], needle: &str) -> bool {
    problems.iter().any(|p| p.contains(needle))
}

#[test]
fn checker_accepts_good_fixtures() {
    let o = "// Expected output: 5\n2 + 3\n// note\n// Expected output: 1\nx:1 // trailing\n";
    assert_eq!(check_o_file("f.o", o), Vec::<String>::new());
    let md = "```\noxidedb> 2 + 3\n5\noxidedb> x:1 // c\n1\n```\n```text\noxidedb> anything\n```\n";
    assert_eq!(check_chapter("f.md", md), Vec::<String>::new());
}

#[test]
fn checker_a_consecutive_expectations() {
    let o = "// Expected output: 5\n// Expected output: 6\n2 + 4\n";
    assert!(reports(&check_o_file("f.o", o), "before any expression"));
}

#[test]
fn checker_b_unclosed_fence() {
    let md = "```\noxidedb> 1\n1\n```\n```\noxidedb> 2\n2\n";
    assert!(reports(&check_chapter("f.md", md), "never closed"));
}

#[test]
fn checker_c_unparsed_prompts_and_skipped_fences() {
    for bad in ["```\noxidedb>1\n1\n```\n", "```\n oxidedb> 1\n1\n```\n"] {
        let md = format!("```\noxidedb> 2\n2\n```\n{}", bad);
        assert!(
            reports(&check_chapter("f.md", &md), "not at the start"),
            "{}",
            bad
        );
    }
    let md = "```\noxidedb> 2\n2\n```\n```bash\noxidedb> 1\n```\n";
    assert!(reports(&check_chapter("f.md", md), "not run"));
}

#[test]
fn checker_d_unusual_fences() {
    for open in ["  ```", "~~~", "````"] {
        let close = open.trim();
        let md = format!(
            "```\noxidedb> 2\n2\n```\n{}\noxidedb> 9\n9\n{}\n",
            open, close
        );
        assert!(
            reports(&check_chapter("f.md", &md), "not run"),
            "{:?}",
            open
        );
    }
}

#[test]
fn checker_e_marker_typos() {
    for typo in [
        "// expected output: 5",
        "//Expected output: 5",
        "// Expected Output: 5",
        "// Expected output:5",
        "// EXPECTED: 5",
        "/ Expected output: 5",
    ] {
        let o = format!("// Expected output: 5\n5\n{}\n5\n", typo);
        assert!(
            reports(&check_o_file("f.o", &o), "looks like an expectation"),
            "{}",
            typo
        );
    }
}

#[test]
fn checker_f_no_truncation_at_parenthesis() {
    let o = "// Expected output: 5 (explanation)\n5\n";
    let p = check_o_file("f.o", o);
    assert!(reports(&p, "expected: 5 (explanation)"), "{:?}", p);
}

#[test]
fn checker_reports_wrong_and_missing_expectations() {
    assert!(reports(
        &check_o_file("f.o", "// Expected output: 6\n2 + 3\n"),
        "actual:   5"
    ));
    assert!(reports(&check_o_file("f.o", "2 + 3\n"), "contains no"));
    assert!(reports(
        &check_chapter("f.md", "```\noxidedb> 2 + 3\n6\n```\n"),
        "actual:   5"
    ));
    assert!(reports(
        &check_chapter("f.md", "no samples\n"),
        "no checked"
    ));
}

#[test]
fn checker_rejects_expected_errors_in_example_files() {
    let o = "// Expected output: 1\n1\n// Expected output: 'type\n1 + \"a\"\n";
    assert!(reports(&check_o_file("f.o", o), "run to completion"));
}

#[test]
fn checker_errors_are_q_style() {
    let md = "```\noxidedb> 1 + \"a\"\n'type\noxidedb> nope\n'nope (Undefined variable)\n```\n";
    assert_eq!(check_chapter("f.md", md), Vec::<String>::new());
    let stale = "```\noxidedb> 1 + \"a\"\nError: Invalid binary operation\n```\n";
    assert!(reports(&check_chapter("f.md", stale), "actual:   'type"));
    // an error with no expectation is a problem
    assert!(reports(
        &check_o_file("f.o", "// Expected output: 1\n1\n1 + \"a\"\n"),
        "failed unexpectedly: 'type"
    ));
}

#[test]
fn checker_multiline_and_empty_output() {
    // a result may span lines; an expression that prints nothing is a prompt with no output
    let md = "```\noxidedb> (1;`a)\n1\n`a\noxidedb> ()\noxidedb> 2\n2\n```\n";
    assert_eq!(check_chapter("f.md", md), Vec::<String>::new());
    // ... and the lines must match exactly, and output must not be missing or surplus
    let wrong = "```\noxidedb> (1;`a)\n1\n`b\n```\n";
    assert!(reports(&check_chapter("f.md", wrong), "actual:   1\n`a"));
    let missing = "```\noxidedb> (1;`a)\noxidedb> 2\n2\n```\n";
    assert!(reports(&check_chapter("f.md", missing), "actual:   1\n`a"));
    let surplus = "```\noxidedb> ()\n1\n```\n";
    assert!(reports(
        &check_chapter("f.md", surplus),
        "expected: 1\n    actual:   "
    ));
}

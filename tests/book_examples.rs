//! Keeps "O for Humans" honest: every expected output in the book is
//! evaluated with the real interpreter.

use oxidedb::{Interpreter, Lexer, Parser};
use std::fs;
use std::path::{Path, PathBuf};

const PROMPT: &str = "oxidedb> ";

/// Evaluate one line and format it the way the REPL prints it.
fn eval(interp: &mut Interpreter, input: &str) -> String {
    let mut run = || -> oxidedb::Result<String> {
        let tokens = Lexer::new(input).tokenize()?;
        let ast = Parser::new(tokens).parse()?;
        Ok(format!("{}", interp.evaluate(ast)?))
    };
    match run() {
        Ok(out) => out,
        Err(e) => format!("Error: {}", e),
    }
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

#[test]
fn book_examples_match_expected_output() {
    let mut failures = Vec::new();
    for path in book_files("examples", "o") {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let text = fs::read_to_string(&path).unwrap();
        let mut interp = Interpreter::new();
        let mut pending: Option<(usize, String)> = None;
        let mut expectations = 0;
        for (i, raw) in text.lines().enumerate() {
            let (n, line) = (i + 1, raw.trim());
            if let Some(rest) = line.strip_prefix("// Expected output:") {
                let rest = rest.trim();
                let expected = rest.split(" (").next().unwrap().trim();
                pending = Some((n, expected.to_string()));
                continue;
            }
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            let actual = eval(&mut interp, line);
            match pending.take() {
                Some((en, expected)) => {
                    expectations += 1;
                    if actual != expected {
                        failures.push(format!(
                            "{}:{}: `{}` (expectation at line {})\n    expected: {}\n    actual:   {}",
                            name, n, line, en, expected, actual
                        ));
                    }
                }
                None if actual.starts_with("Error: ") => {
                    failures.push(format!(
                        "{}:{}: `{}` failed unexpectedly: {}",
                        name, n, line, actual
                    ));
                }
                None => {}
            }
        }
        if let Some((en, _)) = pending {
            failures.push(format!(
                "{}:{}: expectation has no expression after it",
                name, en
            ));
        }
        if expectations == 0 {
            failures.push(format!("{}: contains no `// Expected output:` lines", name));
        }
    }
    assert!(failures.is_empty(), "\n{}\n", failures.join("\n"));
}

#[test]
fn book_chapter_samples_match() {
    let mut failures = Vec::new();
    for path in book_files("chapters", "md") {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let text = fs::read_to_string(&path).unwrap();
        let mut interp = Interpreter::new(); // one fresh session per chapter
        let mut in_fence = false;
        let mut checking = false;
        let mut block: Vec<(usize, &str)> = Vec::new();
        let mut checked = 0;
        for (i, line) in text.lines().enumerate() {
            if let Some(info) = line.trim_end().strip_prefix("```") {
                if in_fence {
                    if checking {
                        checked += check_block(&name, &block, &mut interp, &mut failures);
                        block.clear();
                    }
                } else {
                    // Only unlabelled fences are REPL transcripts; ```text, ```bash etc. are skipped.
                    checking = info.is_empty();
                }
                in_fence = !in_fence;
            } else if in_fence && checking {
                block.push((i + 1, line));
            }
        }
        if checked == 0 {
            failures.push(format!("{}: contains no checked `oxidedb>` samples", name));
        }
    }
    assert!(failures.is_empty(), "\n{}\n", failures.join("\n"));
}

/// Check one transcript block; returns the number of prompts checked.
fn check_block(
    file: &str,
    block: &[(usize, &str)],
    interp: &mut Interpreter,
    failures: &mut Vec<String>,
) -> usize {
    let mut checked = 0;
    let mut idx = 0;
    while idx < block.len() {
        let (n, line) = block[idx];
        idx += 1;
        let Some(input) = line
            .strip_prefix(PROMPT)
            .map(str::trim)
            .filter(|s| !s.is_empty())
        else {
            continue;
        };
        let mut expected = Vec::new();
        while idx < block.len() && !block[idx].1.starts_with(PROMPT.trim_end()) {
            expected.push(block[idx].1);
            idx += 1;
        }
        let expected = expected.join("\n");
        let actual = eval(interp, input);
        checked += 1;
        if actual != expected.trim_end() {
            failures.push(format!(
                "{}:{}: `{}`\n    expected: {}\n    actual:   {}",
                file, n, input, expected, actual
            ));
        }
    }
    checked
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

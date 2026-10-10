use crate::error::{QError, QResult};
use crate::language::interpreter::Interpreter;
use anyhow::{Context, Result};
use rustyline::{error::ReadlineError, DefaultEditor};
use std::io::IsTerminal;
use std::{ffi::OsString, fs, path::PathBuf};

/// The `\\` command ends a session (interactive or script): `\\` alone or followed by
/// whitespace and anything (as in q). Glued text (`\\ls`) is a q system command instead.
fn is_exit_line(line: &str) -> bool {
    line.strip_prefix("\\\\")
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
}

pub struct Repl {
    editor: DefaultEditor,
    interpreter: Interpreter,
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

impl Repl {
    pub fn new() -> Self {
        Self {
            editor: DefaultEditor::new().expect("Failed to create readline editor"),
            interpreter: Interpreter::new(),
        }
    }

    /// History file in the user's home directory, only for interactive sessions.
    fn history_path() -> Option<PathBuf> {
        if !std::io::stdin().is_terminal() {
            return None;
        }
        Self::history_path_from(std::env::var_os("HOME"), std::env::var_os("USERPROFILE"))
    }

    /// `$HOME`, else `%USERPROFILE%` (Windows); empty values count as unset,, plus `.oxidedb_history`.
    pub fn history_path_from(
        home: Option<OsString>,
        userprofile: Option<OsString>,
    ) -> Option<PathBuf> {
        home.filter(|h| !h.is_empty())
            .or(userprofile.filter(|u| !u.is_empty()))
            .map(|h| PathBuf::from(h).join(".oxidedb_history"))
    }

    pub fn run(&mut self) -> Result<()> {
        let history = Self::history_path();
        if let Some(p) = &history {
            let _ = self.editor.load_history(p); // missing file is fine
        }
        loop {
            match self.editor.readline("oxidedb> ") {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    if line == "exit" || line == "quit" || is_exit_line(line) {
                        println!("Goodbye!");
                        break;
                    }

                    let _ = self.editor.add_history_entry(line);

                    match self.eval_line(line) {
                        Ok(Some(result)) => println!("{}", result),
                        Ok(None) => {}
                        Err(e) => eprintln!("{}", e),
                    }
                }
                // Ctrl-C cancels the current line; the session continues.
                Err(ReadlineError::Interrupted) => continue,
                // Ctrl-D (Eof) and any other failure end the session.
                Err(_) => {
                    println!("Goodbye!");
                    break;
                }
            }
        }
        if let Some(p) = &history {
            let _ = self.editor.save_history(p); // unwritable is fine
        }
        Ok(())
    }

    /// Runs a file, with q's script rules (columns matter: markers must start in column 0):
    /// a line of only `\\` or only `\` outside a block ends the script successfully; a line of
    /// only `/` opens a block comment and a line of only `\` closes it (blocks nest; one left
    /// open runs to the end of the file); block bodies are skipped unlexed. Trailing whitespace on
    /// a marker is ignored. The first failing line aborts with an error that wraps the
    /// `QError` in a `line N` context (printed as `line 3: 'type`, and
    /// recoverable with `downcast_ref::<QError>()`). The caller reports it once.
    pub fn run_file(&mut self, filename: &str) -> Result<()> {
        let content =
            fs::read_to_string(filename).with_context(|| format!("cannot read {filename}"))?;
        let content = content.strip_prefix('\u{feff}').unwrap_or(&content);

        let mut block_depth = 0usize;
        for (line_num, raw) in content.lines().enumerate() {
            let marker = raw.trim_end_matches([' ', '\t', '\r']);
            match marker {
                "/" => {
                    block_depth += 1;
                    continue;
                }
                "\\" if block_depth > 0 => {
                    block_depth -= 1;
                    continue;
                }
                _ if block_depth > 0 => continue,
                "\\" => break,
                _ if is_exit_line(marker) => break,
                _ => {}
            }
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }

            let result = self
                .eval_line(line)
                .map_err(|e| anyhow::Error::new(e).context(format!("line {}", line_num + 1)))?;
            if let Some(result) = result {
                println!("{}", result);
            }
        }

        Ok(())
    }

    /// Evaluates one line; `None` for input with no tokens (e.g. a comment).
    pub fn eval_line(&mut self, input: &str) -> QResult<Option<String>> {
        if input.starts_with("\\\\") {
            return Err(QError::Nyi("system command".into()));
        }
        Ok(self.interpreter.eval_line(input)?.map(|v| v.to_string()))
    }
}

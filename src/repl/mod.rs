use crate::language::{interpreter::Interpreter, lexer::Lexer, parser::Parser};
use anyhow::Result;
use colored::*;
use rustyline::{error::ReadlineError, DefaultEditor};
use std::{fs, path::PathBuf};

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

    /// History file in the user's home directory, if `$HOME` is set.
    fn history_path() -> Option<PathBuf> {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".oxidedb_history"))
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

                    if line == "exit" || line == "quit" || line == "\\\\" {
                        println!("Goodbye!");
                        break;
                    }

                    let _ = self.editor.add_history_entry(line);

                    match self.eval_line(line) {
                        Ok(result) => println!("{}", result),
                        Err(e) => eprintln!("{}: {}", "Error".red(), e),
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

    /// Runs a file; the first failing line aborts with an error naming the line.
    /// The caller reports the error (once).
    pub fn run_file(&mut self, filename: &str) -> Result<()> {
        let content = fs::read_to_string(filename)?;
        let content = content.strip_prefix('\u{feff}').unwrap_or(&content);

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            let result = self
                .eval_line(line)
                .map_err(|e| anyhow::anyhow!("at line {}: {}", line_num + 1, e))?;
            println!("{}", result);
        }

        Ok(())
    }

    pub fn eval_line(&mut self, input: &str) -> Result<String> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;

        let result = self.interpreter.evaluate(ast)?;
        Ok(format!("{}", result))
    }
}

use crate::language::{
    interpreter::Interpreter,
    lexer::{Lexer, Token},
    parser::Parser,
};
use anyhow::Result;
use colored::*;
use rustyline::DefaultEditor;
use std::fs;

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

    pub fn run(&mut self) -> Result<()> {
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

                    match self.evaluate(line) {
                        Ok(Some(result)) => println!("{}", result),
                        Ok(None) => {}
                        Err(e) => println!("{}: {}", "Error".red(), e),
                    }
                }
                Err(_) => {
                    println!("Goodbye!");
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn run_file(&mut self, filename: &str) -> Result<()> {
        let content = fs::read_to_string(filename)?;

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            match self.evaluate(line) {
                Ok(Some(result)) => println!("{}", result),
                Ok(None) => {}
                Err(e) => {
                    eprintln!("{} at line {}: {}", "Error".red(), line_num + 1, e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Returns `None` for input with no tokens (e.g. a comment-only line).
    fn evaluate(&mut self, input: &str) -> Result<Option<String>> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        if tokens == [Token::Eof] {
            return Ok(None);
        }

        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;

        let result = self.interpreter.evaluate(ast)?;
        Ok(Some(format!("{}", result)))
    }
}

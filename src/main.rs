use anyhow::Result;
use colored::*;
use oxidedb::repl::Repl;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // File execution mode
        let filename = &args[1];
        println!("{} {}", "OxideDB - Executing".green().bold(), filename);

        let mut repl = Repl::new();
        repl.run_file(filename)
    } else {
        // Interactive REPL mode
        println!(
            "{}",
            "OxideDB - A Q-inspired columnar database".green().bold()
        );
        println!("Type 'exit', 'quit', or '\\\\' to exit\n");

        let mut repl = Repl::new();
        repl.run()
    }
}

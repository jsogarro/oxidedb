use anyhow::Result;
use colored::*;
use oxidedb::repl::Repl;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // File execution mode
        let filename = &args[1];
        // Banner goes to stderr so stdout carries only results.
        eprintln!("{} {}", "OxideDB - Executing".green().bold(), filename);

        let mut repl = Repl::new();
        if let Err(e) = repl.run_file(filename) {
            eprintln!("{}", e);
            std::process::exit(1);
        }
        Ok(())
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

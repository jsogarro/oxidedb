use anyhow::Result;
use colored::*;
use oxidedb::repl::Repl;
use std::env;

/// Stack for the interpreter thread. `evaluate` and the AST drop recurse once per node and the
/// parser bounds the tree to 2,000 levels, which needs about 3 MB in a debug build and under 1 MB in
/// release; 64 MB leaves 20x headroom for raising the limits. Address space is reserved, not committed.
const INTERPRETER_STACK: usize = 64 * 1024 * 1024;

fn main() -> Result<()> {
    // The interpreter's values are `Rc` (not `Send`), so everything is built inside the thread.
    let worker = std::thread::Builder::new()
        .name("interpreter".into())
        .stack_size(INTERPRETER_STACK)
        .spawn(run)?;
    match worker.join() {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // File execution mode
        let filename = &args[1];
        // Banner goes to stderr so stdout carries only results.
        eprintln!("{} {}", "OxideDB - Executing".green().bold(), filename);

        let mut repl = Repl::new();
        if let Err(e) = repl.run_file(filename) {
            eprintln!("{e:#}");
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

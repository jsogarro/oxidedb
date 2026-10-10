# OxideDB (ODB)

An experimental, Q/KDB+-inspired expression interpreter written in Rust. It implements the array programming language **O**; the columnar database engine is planned and not built yet.

## Overview

OxideDB aims to provide the power and expressiveness of Q/KDB+ with Rust's memory safety and performance characteristics. The O language is the query and programming language of OxideDB, featuring Q's right-to-left evaluation and array-oriented design principles.

## Features

- **O Programming Language**: Q-inspired array programming language with right-to-left evaluation
- **Interactive REPL**: Full-featured Read-Eval-Print Loop with command history
- **File Execution**: Run O scripts directly from files (.o extension)
- **Variable System**: Named values held in memory for the length of a session (nothing is saved to disk; REPL input history is kept in `~/.oxidedb_history`)
- **Type System**: Integers, floats, booleans and characters, with null and infinity values (`0N`, `0n`, `0w`) (symbol literals are not implemented yet)
- **Comprehensive Documentation**: "O for Humans" book with executable examples
- **Memory Safety**: Written in safe Rust

## Quick Start

### Prerequisites

- Rust 1.78 or later (`Cargo.lock` is version 4)
- Cargo

### Building

```bash
git clone <repository-url>
cd oxidedb
cargo build --release
```

### Running the REPL

Start the interactive O language environment:
```bash
cargo run
```

### Running O Files

Execute O script files directly:
```bash
# Run example files
cargo run -- book/examples/01-atoms-and-arithmetic.o
cargo run -- book/examples/02-variables-and-assignment.o

# Run any .o file
cargo run -- path/to/your/script.o
```

### Running Tests

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test basics

# Run benchmarks
cargo bench
```

## Current Status

Phase 1 of the O language implementation is **complete** with a fully functional interpreter, REPL, and file execution system.

### Implemented Features

- ✅ **Complete O Language Parser**: Lexer, parser, and AST generation for O syntax
- ✅ **Right-to-Left Evaluation**: Proper Q-style expression evaluation (e.g., `1 + 2 * 3` = `7`)
- ✅ **Variable System**: Assignment (`x:5`) and retrieval, held in memory for the session
- ✅ **Atom Types**: Integers, floats, booleans, characters and symbols with q-style type codes
- ✅ **Vector Literals**: long, float, boolean, symbol and character vectors (strings)
- ✅ **Interactive REPL**: Full-featured environment with command history and error handling
- ✅ **File Execution**: Run .o script files with line-by-line execution and error reporting
- ✅ **Testing**: integration and property tests, plus tests that run every example in the book
- ✅ **Benchmarks**: Criterion benchmarks for basic operations (`cargo bench`)
- ✅ **Documentation**: "O for Humans" book with executable examples

### Roadmap

#### Phase 1: Basic Language ✅ **COMPLETE**
- ✅ Basic atoms (integers, floats, booleans, characters)
- ✅ Simple arithmetic operations with right-to-left evaluation
- ✅ Variable assignment and retrieval (`:` operator)
- ✅ Interactive REPL with file execution support

#### Phase 2: Vectors and Lists (book Chapter 3) 🚧 **IN PROGRESS**
- ✅ Vector literals (`1 2 3`, `1 2.5 3`, `101b`, `` `a`b ``, `"abc"`) and symbol atoms
- ⏳ Vector creation and manipulation (`til`, `count`, take, join)
- ⏳ Indexing and slicing
- ⏳ Basic vector operations (arithmetic, comparison)
- ⏳ Type-preserving operations

#### Phase 3: Dictionaries and Tables (book Chapter 4)
- ⏳ Dictionary implementation
- ⏳ Table as collection of named columns
- ⏳ Basic table operations (select, update, insert)
- ⏳ Simple queries

#### Phase 4: Functions and Control Flow (book Chapter 5)
- ⏳ Function definition and application
- ⏳ Conditionals and loops
- ⏳ Error handling
- ⏳ Adverbs (each, over, scan)

#### Phase 5: Advanced Features
- ⏳ Temporal types and operations
- ⏳ File I/O and persistence (variables are not saved yet)
- ⏳ Inter-process communication
- ⏳ Performance optimizations

## Usage Examples

### Interactive REPL
```bash
$ cargo run
OxideDB - A Q-inspired columnar database
Type 'exit', 'quit', or '\\' to exit

oxidedb> 2 + 3
5
oxidedb> 1 + 2 * 3    // Right-to-left: 1 + (2 * 3) = 7
7
oxidedb> x:10         // Variable assignment
10
oxidedb> y:x*2        // Using variables
20
oxidedb> y
20
oxidedb> 2.5 + 1.5    // Float arithmetic
4f
oxidedb> 1 2.5 3      // A vector literal; one float promotes all
1 2.5 3
oxidedb> \\
Goodbye!
```

### File Execution
```bash
$ cargo run -- book/examples/01-atoms-and-arithmetic.o
42
-17
0
3.14159
-2.5
...
```

## Learning O

### "O for Humans" Book
Complete guide to the O programming language located in `/book/`:
- **Introduction**: Overview of O and OxideDB
- **Chapter 1**: Atoms and Basic Arithmetic
- **Chapter 2**: Variables and Assignment

### Executable Examples
Run interactive examples from the book:
```bash
cargo run -- book/examples/01-atoms-and-arithmetic.o
cargo run -- book/examples/02-variables-and-assignment.o
```

## Architecture

The codebase is organized into several modules:

- `language/`: Lexer, parser, AST, and interpreter for O language
- `types/`: Core data types (atoms, interned symbols and typed columns are implemented; dictionaries and tables are planned)
- `repl/`: Interactive REPL interface with file execution support
- `book/`: Complete "O for Humans" documentation and examples

## Contributing

This project follows Rust best practices and emphasizes:

- **Test-Driven Development**: All features should have comprehensive tests
- **Documentation**: Code should be well-documented with examples
- **Performance**: Benchmarks for performance-critical paths
- **Safety**: Leverage Rust's type system to prevent runtime errors

## Key Differences from Q

While O is inspired by Q, there are some important differences:

- **Language Name**: O (instead of Q)
- **File Extension**: `.o` files (instead of `.q`)
- **Division Operator**: `%` (same as Q): always returns a float, and division by zero follows IEEE 754 (`1%0` is `0w`)
- **Deliberate deviations**: integer overflow is an error (q wraps around), and an out-of-range float literal such as `1e999` is an error. Nulls (`0N`, `0n`) propagate through arithmetic.
- **Exit Commands**: `exit`, `quit`, or `\\` (Q standard)
- **Right-to-Left Evaluation**: Fully implemented like Q

## References

- [Q For Mortals](https://code.kx.com/q4m3/) - Primary reference for Q language features
- [KX Documentation](https://code.kx.com/) - Official Q/KDB+ documentation
- **"O for Humans"** - This project's complete language guide

## License

[License to be determined]

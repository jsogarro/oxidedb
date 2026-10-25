# OxideDB (ODB)

An experimental, Q/KDB+-inspired expression interpreter written in Rust. It implements the array programming language **O**; the columnar database engine is planned and not built yet.

## Overview

OxideDB aims to provide the power and expressiveness of Q/KDB+ with Rust's memory safety and performance characteristics. The O language is the query and programming language of OxideDB, featuring Q's right-to-left evaluation and array-oriented design principles.

## Features

- **O Programming Language**: Q-inspired array programming language with right-to-left evaluation
- **Interactive REPL**: Full-featured Read-Eval-Print Loop with command history
- **File Execution**: Run O scripts directly from files (.o extension)
- **Variable System**: Named values held in memory for the length of a session (nothing is saved to disk; REPL input history is kept in `~/.oxidedb_history`)
- **Type System**: Integers, floats, booleans, characters and symbols, with null and infinity values (`0N`, `0n`, `0w`), and vectors
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
cargo run -- book/examples/03-vectors-and-lists.o

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
- ✅ **Vector Arithmetic**: `+ - * %` item by item, with broadcasting of atoms
- ✅ **Application and indexing**: `til 3+2`, `count til 5`, `v[1]`, `v 0 2`, with q's right-to-left argument rule
- ✅ **Item assignment**: `v[0]:5`, `v[0 2]:7 8`, copy-on-write, exact types (no promotion)
- ✅ **Take and join**: `2#1 2 3` is `1 2`, `1 2,3 4` is `1 2 3 4`
- ✅ **Comparison**: `= <> < <= > >=` on atoms and vectors (`1 2 3 = 1 5 3` is `101b`)
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
- ✅ Several statements on a line (`x:5;x-1`; a trailing `;` prints nothing)
- ✅ Interactive REPL with file execution support

#### Phase 2: Vectors and Lists (book Chapter 3) 🚧 **IN PROGRESS**
- ✅ Vector literals (`1 2 3`, `1 2.5 3`, `101b`, `` `a`b ``, `"abc"`), symbol atoms and vector arithmetic
- ✅ `til`, `count`, `neg` and `enlist`
- ✅ General lists (`(1;`a;2.5)`, `(1 2;3 4)`, `()`): mixed and nested items, normalised to a vector when the items share a type, printed as q does
- ✅ Take and join (`#`, `,`)
- ✅ Indexing (`v[i]`, `v i`, `v[0 2]`; out of range gives a null)
- ✅ Item assignment (`v[i]:x`, `v[0 2]:7 8`; the value must have the vector's type, out of range is `'length`, a shared copy is unchanged)
- ⏳ Slicing
- ✅ Vector arithmetic (`+ - * %` on atoms and vectors)
- ✅ Comparison verbs in the language (`= <> < <= > >=`, giving booleans and boolean vectors)
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
oxidedb> 1 2 3 + 10   // Arithmetic applies to every item
11 12 13
oxidedb> 1 2 3 = 1 5 3   // Comparison gives a boolean vector
101b
oxidedb> v:til 3+2       // A name applies to everything on its right
0 1 2 3 4
oxidedb> v 0 2           // Indexing
0 2
oxidedb> v[9]            // Out of range is a null, not an error
0N
oxidedb> 5#1 2           // Take wraps around
1 2 1 2 1
oxidedb> 1 2,3 4        // Join
1 2 3 4
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
- **Chapter 2**: Variables and Assignment (including several statements on a line with `;`)
- **Chapter 3**: Vectors and Lists (in progress: vector literals, arithmetic, comparison, `til`, `count`, `enlist`, indexing, item assignment, general lists, take and join)

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
- **Division Operator**: `%` (as in q): always returns a float, and division by zero follows IEEE 754 (`1%0` is `0w`)
- **Deliberate deviations**: integer overflow is an error (q wraps around), and an out-of-range float literal such as `1e999` is an error. Nulls (`0N`, `0n`) propagate through arithmetic.
- **Exit Commands**: `\\` is q's; `exit` and `quit` are O conveniences
- **Right-to-Left Evaluation**: as in q
- **Booleans in arithmetic**: `1b+1` is `2` in both; `1b+1b` is an int in q and a long in O
- **Assignment echo**: q prints nothing for `x:5`; O shows the value

## References

- [Q For Mortals](https://code.kx.com/q4m3/) - Primary reference for Q language features
- [KX Documentation](https://code.kx.com/) - Official Q/KDB+ documentation
- **"O for Humans"** - This project's complete language guide

## License

[License to be determined]

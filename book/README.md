# O for Humans

A comprehensive guide to the O programming language and OxideDB database system.

## Table of Contents

- [Introduction](chapters/00-introduction.md)
- [Chapter 1: Atoms and Basic Arithmetic](chapters/01-atoms-and-arithmetic.md)
- [Chapter 2: Variables and Assignment](chapters/02-variables-and-assignment.md)
- [Chapter 3: Vectors and Lists](chapters/03-vectors-and-lists.md) (in progress: vector literals, arithmetic, comparison, take and join so far)

### Coming Soon

- Chapter 4: Dictionaries and Tables  
- Chapter 5: Functions and Control Flow
- Chapter 6: Advanced Operations

## About This Book

"O for Humans" is a practical guide to learning the O programming language, inspired by the classic "Q For Mortals". The O language is the query and programming language of OxideDB, a high-performance columnar database implemented in Rust.

This book teaches O through hands-on examples that you can try in the OxideDB REPL. Each chapter builds upon the previous one, gradually introducing more sophisticated concepts and techniques.

## Getting Started

To follow along with this book, you'll need to have OxideDB installed and running. See the main project README for installation instructions.

Start the REPL with:
```bash
cargo run
```

You can exit the REPL using `exit`, `quit`, or `\\`.

## Running Examples

The `book/examples` directory contains executable O files demonstrating the concepts from each chapter (see [its README](examples/README.md)). Run them with:

```bash
cargo run -- book/examples/01-atoms-and-arithmetic.o
cargo run -- book/examples/02-variables-and-assignment.o
cargo run -- book/examples/03-vectors-and-lists.o
```

Each file includes detailed comments explaining the expected output. `cargo test --test book_examples` evaluates every `// Expected output:` line and every `oxidedb>` sample in the chapters, so the book fails the build when it drifts from the interpreter.
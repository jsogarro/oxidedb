# Introduction to O

Welcome to "O for Humans", your guide to the O programming language and OxideDB database system. 

## What is O?

O is a powerful array programming language inspired by Q/KDB+, designed for high-performance data analysis and manipulation. It's the native language of OxideDB, a columnar database built with Rust's memory safety and performance characteristics.

## Why Learn O?

O combines the expressiveness of Q with modern language design principles:

- **Concise Syntax**: Express complex operations in few characters
- **Right-to-Left Evaluation**: Natural mathematical evaluation order  
- **Array-Oriented**: Designed for working with collections of data (Chapter 3 starts on vectors: a whole list of values in one go)
- **Interactive**: Immediate feedback through the REPL
- **Strict About Types**: Combinations that make no sense are errors, not silent conversions (for example, `1 + "a"`)

## The OxideDB Environment

OxideDB provides an interactive environment where you can:
- Execute O expressions immediately
- Define variables and reuse their values

Vector literals, general lists (`(1;`a)`), vector arithmetic, comparisons, `til`, `count`, `neg`, `enlist`, indexing, take (`#`) and join (`,`) work (Chapter 3). Tables and queries are still to come.

## Getting Started

Launch the OxideDB REPL:

```bash
cargo run
```

You'll see:
```text
OxideDB - A Q-inspired columnar database
Type 'exit', 'quit', or '\\' to exit

oxidedb> 
```

Try your first O expression:
```
oxidedb> 2 + 3
5
```

## Conventions Used in This Book

- **Code blocks** show O expressions you can type in the REPL
- **Results** are shown immediately after the expression
- **Comments** start with `/` at the beginning of a line or after a space (`//` works too) and run to the end of the line; they explain what's happening
- **Script files** (`cargo run -- file.o`) follow q's file rules, starting in column 0: a line holding only `/` begins a block comment and a line holding only `\` ends it (blocks nest, an unclosed one runs to the end of the file, and the lines inside are skipped without being read); a line holding only `\\`, or a `\` outside any block, ends the script successfully and skips the rest. These rules are for files only: the REPL treats a lone `/` as an ordinary comment. They cannot be shown as REPL transcripts, so they are checked by `tests/scripts.rs`, which runs the binary on script files
- **Exercises** appear at the end of each chapter

## Key Concepts

Before diving in, understand these fundamental concepts:

1. **Atoms**: Individual values (numbers, booleans, characters and symbols)
2. **Right-to-Left Evaluation**: `2 + 3 * 4` equals `14`, not `10`
3. **Variables**: Store values using `:` (e.g., `x:5`)
4. **Types**: O is dynamically typed. The type belongs to the value, not the variable, so `x:5` followed by `x:2.5` is fine

Let's begin your journey into the world of O!
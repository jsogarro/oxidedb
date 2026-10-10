# Introduction to O

Welcome to "O for Humans", your guide to the O programming language and OxideDB database system. 

## What is O?

O is a powerful array programming language inspired by Q/KDB+, designed for high-performance data analysis and manipulation. It's the native language of OxideDB, a columnar database built with Rust's memory safety and performance characteristics.

## Why Learn O?

O combines the expressiveness of Q with modern language design principles:

- **Concise Syntax**: Express complex operations in few characters
- **Right-to-Left Evaluation**: Natural mathematical evaluation order  
- **Array-Oriented**: Built for working with collections of data
- **Interactive**: Immediate feedback through the REPL
- **Type Safe**: Leverages Rust's type system for reliability

## The OxideDB Environment

OxideDB provides an interactive environment where you can:
- Execute O expressions immediately
- Store and manipulate data in memory
- Define and use variables
- Build complex data analysis workflows

## Getting Started

Launch the OxideDB REPL:

```bash
cargo run
```

You'll see:
```
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
- **Exercises** appear at the end of each chapter

## Key Concepts

Before diving in, understand these fundamental concepts:

1. **Atoms**: Individual values (numbers, characters, symbols)
2. **Right-to-Left Evaluation**: `2 + 3 * 4` equals `14`, not `10`
3. **Variables**: Store values using `:` (e.g., `x:5`)
4. **Types**: O is strongly typed with automatic inference

Let's begin your journey into the world of O!
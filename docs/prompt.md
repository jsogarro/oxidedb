# OxideDB Project Setup Prompt

## Project Overview
Create a new Rust project called OxideDB - a high-performance columnar database with an integrated array programming language inspired by Q/KDB+. The implementation should follow the structure and functionality described in "Q For Mortals" (available free online), implementing features chapter by chapter with comprehensive test coverage.

## Initial Project Structure

```
oxidedb/
├── Cargo.toml
├── README.md
├── .gitignore
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── language/
│   │   ├── mod.rs
│   │   ├── lexer.rs
│   │   ├── parser.rs
│   │   ├── ast.rs
│   │   └── interpreter.rs
│   ├── types/
│   │   ├── mod.rs
│   │   ├── atom.rs
│   │   ├── vector.rs
│   │   ├── dictionary.rs
│   │   └── table.rs
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── memory.rs
│   │   ├── storage.rs
│   │   └── query.rs
│   └── repl/
│       └── mod.rs
├── tests/
│   ├── integration/
│   └── unit/
├── benches/
│   └── performance.rs
└── docs/
    └── architecture.md
```

## Core Requirements

1. **Language Implementation**
   - Create a lexer that tokenizes Q-like syntax
   - Build a parser that generates an AST
   - Implement an interpreter for basic expressions
   - Support for basic types: atoms, vectors, dictionaries, and tables
   - REPL (Read-Eval-Print Loop) for interactive development
   - Make sure the language is strongly typed but mainly relies on type inference.
   - The syntax should exactly match Q and qSQL.

2. **Type System**
   - Implement Q's type system with proper type codes
   - Support for temporal types (date, time, timestamp)
   - Null values for each type
   - Type promotion rules

3. **Memory Model**
   - Columnar storage for vectors and tables
   - Reference counting for memory management
   - Copy-on-write semantics where appropriate

4. **Testing Strategy**
   - Unit tests for each module with 100% coverage target
   - Integration tests for language features
   - Property-based testing for core operations
   - Benchmark tests for performance-critical paths

## Initial Cargo.toml

```toml
[package]
name = "oxidedb"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core dependencies
anyhow = "1.0"
thiserror = "1.0"
chrono = "0.4"

# REPL dependencies
rustyline = "14.0"
colored = "2.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"

# Performance
rayon = "1.7"
parking_lot = "0.12"

[dev-dependencies]
criterion = "0.5"
proptest = "1.0"
pretty_assertions = "1.4"

[[bench]]
name = "performance"
harness = false
```

## Implementation Roadmap

### Phase 1: Basic Language (Chapters 1-3 of Q For Mortals)
- [ ] Basic atoms (integers, floats, booleans, characters)
- [ ] Simple arithmetic operations
- [ ] Variable assignment and retrieval
- [ ] Basic REPL functionality

### Phase 2: Vectors and Lists (Chapter 4)
- [ ] Vector creation and manipulation
- [ ] Indexing and slicing
- [ ] Basic vector operations (arithmetic, comparison)
- [ ] Type-preserving operations

### Phase 3: Dictionaries and Tables (Chapters 5-6)
- [ ] Dictionary implementation
- [ ] Table as collection of named columns
- [ ] Basic table operations (select, update, insert)
- [ ] Simple queries

### Phase 4: Functions and Control Flow (Chapters 7-8)
- [ ] Function definition and application
- [ ] Conditionals and loops
- [ ] Error handling
- [ ] Adverbs (each, over, scan)

### Phase 5: Advanced Features
- [ ] Temporal types and operations
- [ ] File I/O and persistence
- [ ] Inter-process communication
- [ ] Performance optimizations

## Testing Guidelines
1
1. **Unit Tests**: Each public function should have corresponding tests
2. **Integration Tests**: Test complete workflows and language features
3. **Documentation Tests**: All examples in documentation should be executable
4. **Coverage**: Maintain >95% test coverage, use `cargo tarpaulin` for measurement

## Development Workflow

1. Create feature branch for each Q For Mortals chapter/section
2. Implement functionality with tests first (TDD)
3. Ensure all tests pass and coverage is maintained
4. Document new features in code and update architecture docs
5. Create benchmarks for performance-critical features

## Initial Files to Create

1. **src/main.rs**: Entry point with basic REPL
2. **src/language/lexer.rs**: Tokenizer for Q-like syntax
3. **src/types/atom.rs**: Basic atomic types
4. **tests/integration/basics.rs**: Integration tests for basic functionality
5. **README.md**: Project description and build instructions

## Notes

- Use Q For Mortals as a functional specification, not for code copying
- Focus on Rust idioms and safety while maintaining Q's performance characteristics
- Prioritize correctness over performance in initial implementation
- Keep modules small and focused on single responsibilities
- Use Rust's type system to prevent runtime errors where possible

Please set up this project structure and implement the basic REPL with support for integer arithmetic as the first milestone.
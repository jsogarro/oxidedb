# OxideDB Architecture

## Overview

OxideDB is designed as a modular system with clear separation of concerns between language processing, data storage, and query execution.

## Module Structure

### Language Module (`src/language/`)

- **Lexer** (`lexer.rs`): Tokenizes Q-like syntax into a stream of tokens
- **Parser** (`parser.rs`): Builds an Abstract Syntax Tree (AST) from tokens
- **AST** (`ast.rs`): Defines the structure of parsed expressions
- **Interpreter** (`interpreter.rs`): Evaluates AST nodes and executes operations

### Types Module (`src/types/`)

- **Atom** (`atom.rs`): Scalar values with Q's type system
- **Vector** (`vector.rs`): Homogeneous arrays of atoms
- **Dictionary** (`dictionary.rs`): Key-value mappings
- **Table** (`table.rs`): Columnar data structures

### Planned

Persistent storage, a query planner, and memory management are not implemented yet.

### REPL Module (`src/repl/`)

- Interactive Read-Eval-Print Loop implementation
- Command history and line editing
- Error handling and display

## Design Principles

### Type System

OxideDB implements Q's type system with the following type codes:

- `-1`: Boolean
- `-7`: Long (64-bit integer)
- `-9`: Float (64-bit float)
- `-10`: Character
- `-11`: Symbol
- `-12`: Timestamp
- `-14`: Date
- `-19`: Time

Each type has corresponding null values and promotion rules.

### Memory Model

- **Reference Counting**: Shared data structures use Arc<RwLock<T>>
- **Copy-on-Write**: Immutable operations avoid unnecessary copying
- **Columnar Storage**: Tables store data by column for cache efficiency

### Error Handling

- Uses `anyhow::Result<T>` for comprehensive error propagation
- Graceful error recovery in the REPL
- Detailed error messages with context

## Future Considerations

### Performance Optimizations

- SIMD operations for vector arithmetic
- Just-in-time compilation for hot code paths
- Memory-mapped file I/O for large datasets

### Concurrency

- Lock-free data structures where possible
- Work-stealing thread pool for parallel operations
- Async I/O for network operations

### Compatibility

- Q language compatibility where feasible
- Import/export compatibility with KDB+ formats
- SQL interface for broader accessibility
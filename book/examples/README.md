# O Language Examples

This directory contains executable O files (.o extension) demonstrating the concepts covered in "O for Humans". Each file corresponds to a chapter and includes commented examples with expected output.

## How to Run Examples

To execute an O file:
```bash
cargo run -- examples/filename.o
```

Or from the project root:
```bash
cargo run -- book/examples/filename.o
```

## Available Examples

- `01-atoms-and-arithmetic.o` - Basic atoms and arithmetic operations
- `02-variables-and-assignment.o` - Variable assignment and usage

## File Format

Each .o file contains:
- Comments explaining the concepts (lines starting with `//`)
- O expressions to execute
- Expected output documented in comments above each expression

The O interpreter will:
- Skip comment lines and empty lines
- Execute each O expression line by line
- Display the result of each expression
- Stop execution if an error occurs

## Example Output

When you run an example file, you'll see output like:
```
$ cargo run -- book/examples/01-atoms-and-arithmetic.o
OxideDB - Executing book/examples/01-atoms-and-arithmetic.o
42
-17
0
3.14159
...
```

Each line of output corresponds to the result of executing an O expression from the file.
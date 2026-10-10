# O Language Examples

This directory contains executable O files (.o extension) demonstrating the concepts covered in "O for Humans". Each file corresponds to a chapter and includes commented examples with expected output.

## How to Run Examples

To execute an O file, from the project root:
```bash
cargo run -- book/examples/filename.o
```

## Available Examples

- `01-atoms-and-arithmetic.o` - Basic atoms and arithmetic operations
- `02-variables-and-assignment.o` - Variable assignment and usage
- `03-vectors-and-lists.o` - Vector literals (longs, floats, booleans, symbols, strings), arithmetic on vectors, and general lists

## File Format

Each .o file contains:
- Comments explaining the concepts (`/` at the start of a line or after whitespace comments out the rest of the line, so `//` works too and trailing comments are allowed; a `/` glued to the previous token is the over adverb)
- Block comments (file mode only, markers in column 0): a line with only `/` starts one, a line with only `\` ends it; blocks nest and an unclosed block runs to the end of the file. A line with only `\\` (or a `\` outside a block) ends the script successfully. `cargo test --test scripts` checks these by running the binary; the example checker `book_examples` reads one line at a time, so example files do not use them
- O expressions to execute
- Expected output documented in comments above each expression

The O interpreter will:
- Skip comment-only and empty lines
- Execute each O expression line by line
- Display the result of each expression
- Stop at the first error: it is reported once on stderr (q style, with the line number: `line 3: 'type`) and the exit status is 1
- Write the banner to stderr, so stdout contains only results (and a leading UTF-8 BOM is ignored)

## Example Output

When you run an example file, you'll see output like:
```text
$ cargo run -- book/examples/01-atoms-and-arithmetic.o
42
-17
0
3.14159
-2.5
...
```

Each line of output corresponds to the result of executing an O expression from the file.

## Checking the Examples

`cargo test --test book_examples` runs each file in its own interpreter session. The next line that prints something after a `// Expected output: X` comment must print exactly `X`. Put any explanation on a separate comment line, and keep the marker in that exact form: a misspelled marker fails the test.

Example files contain only lines that succeed: they must run to completion (`cargo run -- book/examples/NN-name.o` exits 0), and the test fails if an expected result is an error. Error samples live only in the chapter transcripts, which are checked too.
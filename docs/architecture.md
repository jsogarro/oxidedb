# OxideDB Architecture

OxideDB is currently a small q-inspired expression interpreter (the **O** language) with a REPL and file runner. Everything described outside the "Planned" section exists on `main`.

## Modules

### `src/language/`

- `lexer.rs`: turns a line of text into a `Vec<Token>` ending in `Token::Eof`. Handles integer and float literals, `1b`/`0b` booleans, `"c"` characters, `"abc"` strings (with `\" \\ \n \t \r` escapes), `` `a ``/`` `a`b `` symbols, `101b` boolean vectors, the comparison and punctuation tokens `= < > <> <= >= # , ! { } $ @ ' ':` (the parser reports these as not yet implemented), identifiers (variable names), the `0N`/`0n`/`0w` null and infinity literals, and `/` comments (a `/` at line start or after whitespace comments out the rest of the line).
- `parser.rs`: builds an `Expr` from the tokens.
- `ast.rs`: `Expr` (atom, symbol, binary op, unary op, assignment) and the `+ - * %` / negate operators.
- `interpreter.rs`: evaluates an `Expr` against a `HashMap<String, Atom>` of variables. `Interpreter::eval_line` runs the whole pipeline for one line.

### `src/types/`

- `atom.rs`: the scalar `Atom` enum (boolean, long, float, character, symbol, date, time, timestamp, and typed temporal nulls) with q type codes and q-style display.
- `sym.rs`: interned symbols, a 4-byte handle into a process-global string table. Interned names are never freed.
- `column.rs`: `Column`, a typed vector (bool, long, float, char, symbol). It exists as a type with its own tests but the language cannot create one yet.
- `display.rs`: q-style `Display` for `Column` (`1 2 3`, `1 2 3f`, `101b`, `` `a`b ``, `"abc"`) and the character escaping shared with `Atom`'s `Display`.

### `src/repl/` and `src/main.rs`

`Repl` wraps a `rustyline` editor and one `Interpreter`. `main` starts the interactive loop, or runs a file when given a path argument.

## Evaluation pipeline

1. **Tokenize**: `Lexer::tokenize`. A line with no tokens other than `Eof` (blank or comment-only) evaluates to nothing.
2. **Parse**: `Parser::parse`. A flat operator chain is parsed iteratively into a right-nested tree, so there is no precedence: `1+2*3` is `1+(2*3)`. Limits keep recursion bounded: nesting of parentheses and monadic minus is capped at 128, and a flat chain at 2,000 operators. Exceeding either is an error, not a crash.
3. **Evaluate**: `Interpreter::evaluate`. The right operand is evaluated before the left, as in q, including side effects such as assignment. Mixed long/float arithmetic promotes to float; `%` is always float division.

## Values

Every result is an `Atom`. Symbol, date, time and timestamp atoms exist as types, but the language has no literal for them yet. Long, float, character and symbol nulls are sentinels, not separate variants: `Integer(i64::MIN)`, `Float(NaN)`, `Character(' ')`, `Symbol(Sym::NULL)`. Producing `i64::MIN` by arithmetic is reported as overflow. Only the temporal nulls have their own variants. Variables live in the interpreter's map for the length of the session and are not saved.

## Error handling

Lexing, parsing and evaluation return `Result<_, QError>` (`src/error.rs`). `QError` is a q-style error whose text is a quote and a short name: `'type`, `'overflow`, `'parse: <detail>`, `'<name> (Undefined variable)`, `'nyi: <detail>`; `length`, `rank`, `index`, `domain`, `stack` and `signal` are defined for later features. Malformed input and the limits above are `Parse`, features O does not have yet (`0W`, adverbs, strings, symbols) are `Nyi`, unsupported operand types are `Type`, and integer overflow is `Overflow`. Tokens appear in messages in source form (`'parse: unexpected -1 after expression`). The REPL prints the error text on stderr and keeps the session; `run_file` stops at the first failing line and prints the error once with its line number (`line 3: 'type`); the `QError` stays the source of the returned `anyhow` error, and the process exits with status 1.

## Testing

- `tests/*.rs`: one integration test file per area (arithmetic, atoms, comments, display, literals, nesting limits, parsing, types, file running). They drive the public API (`Interpreter`, `Lexer`, `Parser`).
- `tests/robustness.rs`: property tests (`proptest`) asserting that arbitrary input never panics.
- `tests/book_examples.rs`: runs every expected output in `book/` and `book/examples/` through the real interpreter.
- `benches/performance.rs`: Criterion benchmarks for the lexer, parser, evaluator and the full line pipeline.

There are no `#[cfg(test)]` modules in `src/`.

## Planned

Not implemented; nothing here is partially present.

- Vectors and lists in the language (indexing, vector arithmetic).
- Dictionaries and tables, and queries over them.
- Functions, conditionals and adverbs.
- Persistence of variables and tables.

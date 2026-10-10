# OxideDB Architecture

OxideDB is currently a small q-inspired expression interpreter (the **O** language) with a REPL and file runner. Everything described outside the "Planned" section exists on `main`.

## Modules

### `src/language/`

- `lexer.rs`: turns a line of text into a `Vec<Token>` ending in `Token::Eof`. Handles integer and float literals, `1b`/`0b` booleans, `"c"` characters, `"abc"` strings (with `\" \\ \n \t \r` escapes), `` `a ``/`` `a`b `` symbols, `101b` boolean vectors, the comparison and punctuation tokens `= < > <> <= >= # , ! { } $ @ ' ': /: \: ::` (the parser reports these as not yet implemented), identifiers (variable names; a literal glued to another, as in `1.5.5` or `"ab""cd"`, is an invalid literal), the `0N`/`0n`/`0w` null and infinity literals, and `/` comments (a `/` at line start or after whitespace comments out the rest of the line).
- `parser.rs`: builds an `Expr` from the tokens.
- `ast.rs`: `Expr` (atom, symbol, binary op, unary op, assignment) the negate operator, and `Verb`, the binary verbs `+ - * % = < > <> <= >= # , !`. The parser produces only `+ - * %`; the comparison verbs evaluate when an `Expr` is built by hand, and `# , !` are placeholders that evaluate to `'nyi`.
- `interpreter.rs`: evaluates an `Expr` against a `HashMap<String, Value>` of variables. `Interpreter::eval_line` runs the whole pipeline for one line. Verbs are applied by `ops`.
- `builtins.rs`: `lookup(name)` resolves a keyword to a `Builtin` (`name`, `arity`, `call`); `call(name, args)` wraps it. Keywords: `til` and `count`. `til` and `Column::take` share the `MAX_ELEMS` cap (10,000,000 elements, `'domain` beyond it). The interpreter does not resolve names to them yet.
- `ops/`: verb kernels over `Value`s. `ops::dyad(verb, &l, &r)` and `ops::monad_neg(&v)` are the entry points. `ops/arith.rs` implements `+ - * %` atomically: atoms, vectors (broadcast, or pairwise with equal lengths) and general lists (item-wise, renormalised through `Value::from_items`). A boolean counts as a long, long with float is float, `%` is always float, a long null stays null (`0n` once a float is involved), and any element overflowing is `'overflow`. Symbols, characters and temporals are `'type`; a length mismatch is `'length`. `ops/compare.rs` implements `= <> < <= > >=` with the same broadcasting and length rules, returning booleans (a `Bool` column for vectors): bool, long and float compare by value, null equals null and sorts lowest, symbols compare by name, characters by code, and mixing kinds (symbol or character with a number, or with each other) is `'type`. Deliberate deviations from q: comparing a character with a number is `'type` (q compares by code), and float `=` is exact (q is tolerant); a long against a float compares after conversion to float, as in q, so above 2^53 distinct longs can equal a float. `ops::atomic` is the shared list recursion. Verbs without a kernel (`# , !`) give `'nyi: <verb>`.

### `src/types/`

- `atom.rs`: the scalar `Atom` enum (boolean, long, float, character, symbol, date, time, timestamp, and typed temporal nulls) with q type codes and q-style display.
- `sym.rs`: interned symbols, a 4-byte handle into a process-global string table. Interned names are never freed.
- `column.rs`: `Column`, a typed vector (bool, long, float, char, symbol). It is reachable through `Value::Vector`, but the language cannot create one yet. `index` (typed gather, typed nulls for bad indices), `take` (q `#`, cyclic, negative from the end) and `concat` (same type only) are the helpers indexing, take and join build on.
- `value.rs`: `Value`, the result of evaluation (atom, vector or general list), its equality and display.
- `display.rs`: q-style `Display` for `Column` (`1 2 3`, `1 2 3f`, `101b`, `` `a`b ``, `"abc"`) and the character escaping shared with `Atom`'s `Display`.

### `src/repl/` and `src/main.rs`

`Repl` wraps a `rustyline` editor and one `Interpreter`. `main` starts the interactive loop, or runs a file when given a path argument.

## Evaluation pipeline

1. **Tokenize**: `Lexer::tokenize`. A line with no tokens other than `Eof` (blank or comment-only) evaluates to nothing.
2. **Parse**: `Parser::parse`. A flat operator chain is parsed iteratively into a right-nested tree, so there is no precedence: `1+2*3` is `1+(2*3)`. Limits keep recursion bounded: nesting of parentheses and monadic minus is capped at 128, and a flat chain at 2,000 operators. Exceeding either is an error, not a crash.
3. **Evaluate**: `Interpreter::evaluate`. The right operand is evaluated before the left, as in q, including side effects such as assignment. Mixed long/float arithmetic promotes to float; `%` is always float division.

## Values

Evaluation returns a `Value` (`src/types/value.rs`): an `Atom`, a `Vector(Rc<Column>)` (typed, one element type) or a `List(Rc<Vec<Value>>)` (general list, type code 0). Atoms have negative type codes, vectors positive, general lists 0. `Value::from_items` is the one constructor for lists: items that are all atoms of one column type become a `Vector` (no int to float promotion); empty input or anything else is a `List`. Variables hold `Value`s, so reading a vector variable is a reference-count bump. `Interpreter::set` and `get` bind and read variables from the library API.

Equality treats nulls as equal (`0n` equals `0n`, in atoms, columns and lists), while `0f` and `-0f` stay equal. `Value` also compares with `Atom` in both directions, so an atom result can be asserted directly. A general list prints one item per line, `()` when empty.

Arithmetic is atomic: `+ - * %` and unary minus extend over vectors and lists (see `ops/arith.rs` above), though the language cannot build a vector yet, so this is reachable only through `Interpreter::set` and the library API. The parser produces the `Verb`s `+ - * %`; `= < > <> <= >=` evaluate through `ops/compare.rs` but are not parsed yet (`1=1` is `'nyi: =`); `# , !` exist in the AST and give `'nyi: <verb>` until implemented.

Symbol, date, time and timestamp atoms exist as types, but the language has no literal for them yet. Long, float, character and symbol nulls are sentinels, not separate variants: `Integer(i64::MIN)`, `Float(NaN)`, `Character(' ')`, `Symbol(Sym::NULL)`. Producing `i64::MIN` by arithmetic is reported as overflow. Only the temporal nulls have their own variants. Variables live in the interpreter's map for the length of the session and are not saved.

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

- Vector and list literals and indexing (the `Value` type exists; the language cannot yet build one).
- Dictionaries and tables, and queries over them.
- Functions, conditionals and adverbs.
- Persistence of variables and tables.

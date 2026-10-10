# Chapter 1: Atoms and Basic Arithmetic

In O, as in Q, everything starts with atoms - the fundamental building blocks of data. An atom is a single value that cannot be divided further.

## Types of Atoms

O supports several types of atomic values:

### Integers
Whole numbers, positive or negative:
```
oxidedb> 42
42
oxidedb> -17
-17
oxidedb> 0
0
```

### Floating Point Numbers
Numbers with decimal places (the leading zero is optional, so `.5` is `0.5`):
```
oxidedb> 3.14159
3.14159
oxidedb> -2.5
-2.5
oxidedb> .5
0.5
oxidedb> 0.0
0f
```

### Float Display
O prints a float with up to 7 significant digits. A float with no fractional part gets an `f` suffix so you can tell it from an integer (`20f`, `0f`). Very large and very small values switch to e-notation:
```
oxidedb> 20.0
20f
oxidedb> 0.1 + 0.2
0.3
oxidedb> 0.00001
1e-05
oxidedb> 12345678.0
1.234568e+07
```

Scientific notation uses a lowercase `e` followed by an optional sign and digits, and an `f` suffix marks a whole number as a float. The result is always a float:
```
oxidedb> 1e3
1000f
oxidedb> 1.5e3
1500f
oxidedb> 1e-3
0.001
oxidedb> 2f
2f
```
O rejects a bare `1e` or `1e+` (in q `1e` is a real-typed literal), and so is a literal too large for a float (`1e999`: `'parse: float out of range`).

### Nulls and infinities
`0N` is the long (integer) null, `0n` is the float null (it also shows up as the result of a calculation that is not a number), and `0w` and `-0w` are the float infinities. They display exactly as typed:
```
oxidedb> 0N
0N
oxidedb> 0n
0n
oxidedb> 0w
0w
oxidedb> -0w
-0w
```
`0N` is an integer and the other three are floats. A null in arithmetic gives a null: a long null with another long gives `0N`, and a long null mixed with a float (or divided with `%`) gives `0n`:
```
oxidedb> 0N + 1
0N
oxidedb> 0N + 1.5
0n
oxidedb> 0N % 2
0n
```
`0N` is stored as the smallest 64-bit integer, so O reads the literal `-9223372036854775808` as `0N` (q rejects that literal; its long null is written `0N`). See the overflow section for what happens when a computation lands on that value. `0W` (the long infinity) is not supported: it gives `'nyi: 0W (long infinity)`.

Float arithmetic that grows past the largest float gives an infinity, but a float *literal* that is out of range is an error:
```
oxidedb> 1e308 * 10
0w
oxidedb> 1e999
'parse: float out of range: 1e999
```

### Booleans
True or false values:
```
oxidedb> 1b
1b
oxidedb> 0b
0b
```

In arithmetic a boolean counts as a long, `0` or `1`, so the result is a long. (q agrees that `1b+1` is `2`, but gives an int for `1b+1b` where O gives a long.) With a float the result is a float:
```
oxidedb> 1b + 1
2
oxidedb> 1b + 1b
2
oxidedb> 2 * 1b
2
oxidedb> 1b + 0.5
1.5
oxidedb> -1b
-1
```

### Characters
Single characters enclosed in double quotes (several characters in quotes make a string, which is a list of characters; see Chapter 3):
```
oxidedb> "a"
"a"
oxidedb> "Z"
"Z"
oxidedb> "5"
"5"
```

A backslash escapes the next character: `\n`, `\t`, `\r`, `\\` and `\"` each give a single character, and the REPL prints them in the same escaped form:
```
oxidedb> "\n"
"\n"
oxidedb> "\""
"\""
```

## Basic Arithmetic Operations

O supports the four fundamental arithmetic operations:

### Addition (+)
```
oxidedb> 2 + 3
5
oxidedb> 10 + 7
17
oxidedb> 2.5 + 1.5
4f
```

### Subtraction (-)
```
oxidedb> 10 - 3
7
oxidedb> 5 - 8
-3
oxidedb> 3.7 - 1.2
2.5
```

### Multiplication (*)
```
oxidedb> 4 * 5
20
oxidedb> 3 * 7
21
oxidedb> 2.5 * 4
10f
```

### Division (%)
In O, like Q, division uses the `%` symbol and **always returns a float**, even when both operands are integers:
```
oxidedb> 15 % 3
5f
oxidedb> 20 % 4
5f
oxidedb> 7 % 2
3.5
```

Dividing by zero is not an error; it follows IEEE 754, so `1 % 0` is `0w` (infinity) and `0 % 0` is `0n` (the float null, shown for results that are not a number).

The `/` character is **not** division. After a space it starts a comment, so everything from it to the end of the line is ignored; glued to the previous token it is the *over* adverb, which O does not implement yet:
```
oxidedb> 6 / 2
6
oxidedb> 6/2
'nyi: adverb '/'
oxidedb> 6 % 2
3f
```
To divide, always use `%`.

### Overflow
Integers are 64-bit. Integer arithmetic that leaves that range is an error, never a silent wrap-around:
```
oxidedb> 9223372036854775807 + 1
'overflow
```

The smallest 64-bit value, `-9223372036854775808`, is reserved as the integer null, which prints as `0N`. So O reads that literal as a null (q rejects it), and an operation that would land exactly on it is an overflow error:
```
oxidedb> -9223372036854775808
0N
oxidedb> -9223372036854775807 - 1
'overflow
```

A null operand is not an overflow: arithmetic on `0N` gives `0N`, so `-9223372036854775808 + 1` is `0N`:
```
oxidedb> -9223372036854775808 + 1
0N
```

## Right-to-Left Evaluation

**This is crucial**: O evaluates expressions from right to left, unlike most programming languages. This means operations are applied in the order they appear when reading from right to left.

### Understanding Right-to-Left
```
oxidedb> 1 + 2 * 3
7
```

This is evaluated as `1 + (2 * 3)` = `1 + 6` = `7`, **not** as `(1 + 2) * 3` = `3 * 3` = `9`.

More examples:
```
oxidedb> 2 + 3 * 4
14
```
Evaluated as: `2 + (3 * 4)` = `2 + 12` = `14`

```
oxidedb> 4 - 2 + 1
1
```
Evaluated as: `4 - (2 + 1)` = `4 - 3` = `1`

```
oxidedb> 8 % 2 + 2
2f
```
Evaluated as: `8 % (2 + 2)` = `8 % 4` = `2`

Right-to-left also decides which side runs first when an assignment is involved. The right operand is evaluated before the left one, so in `x+x:2` the assignment happens first (assignment is covered in Chapter 2):
```
oxidedb> x:1
1
oxidedb> x+x:2
4
```

### Using Parentheses
When you need to override the right-to-left evaluation, use parentheses:
```
oxidedb> (2 + 3) * 4
20
```
This forces the addition to happen first: `(2 + 3) * 4` = `5 * 4` = `20`

```
oxidedb> (10 - 3) * 2
14
```

## Mixed Type Arithmetic

O automatically handles mixed-type arithmetic by promoting integers to floats when needed:

```
oxidedb> 5 + 2.5
7.5
oxidedb> 3.0 * 4
12f
oxidedb> 10 % 3.0
3.333333
```

## Negative Numbers

Use the minus sign to create negative numbers:
```
oxidedb> -5
-5
oxidedb> -3.14
-3.14
oxidedb> -5 + 3
-2
```

A minus sign glued to a number, with a space before it, is a negative literal, not subtraction. As in q, `2 -1` is therefore the two-item list `2 -1`, not subtraction (lists are the subject of Chapter 3). Write `2 - 1` or `2-1` for subtraction:
```
oxidedb> 2 - 1
1
oxidedb> 2-1
1
oxidedb> 2 -1
2 -1
```

O lets you negate an expression with a leading minus: a minus applied to an expression (rather than glued to a number) negates everything to its right, so `-x+3` is `-(x+3)`. (q has no prefix minus for expressions; it uses the keyword `neg`, and `neg x+3` gives the same value.)
```
oxidedb> x:2
2
oxidedb> -x+3
-5
```

A minus glued to a digit is part of the number, so `-5 + 3` is `-2`; with a space, `- 5 + 3` applies O's leading minus to `5 + 3` and gives `-8`:
```
oxidedb> -5 + 3
-2
oxidedb> - 5 + 3
-8
```

## Reading Errors

O follows q's style for errors: a quote followed by a short name (`'type`, `'length`). Some names and details are O's own additions: q has no `'parse` or `'overflow` error (it wraps on overflow), prints only `'name` for an undefined name, and the `(Undefined variable)` note and the `'nyi: ...` details are O's. Each line below is one kind of error:
```
oxidedb> 1 + "a"
'type
oxidedb> 9223372036854775807 + 1
'overflow
oxidedb> 1 +
'parse: unexpected end of input
oxidedb> nope
'nope (Undefined variable)
oxidedb> 6/2
'nyi: adverb '/'
```

- `'type`: the operands do not fit the operation
- `'overflow`: an integer result left the 64-bit range
- `'parse: ...`: the text is not valid O; the detail says what the parser or lexer saw
- `'x (Undefined variable)`: the name `x` has not been assigned
- `'nyi: ...`: not yet implemented; the detail names the missing feature, such as an adverb

The error replaces the result and the session carries on. When you run a file, the first error stops the run and is printed once with its line number, for example `line 3: 'type`.

## Exercises

Try these expressions in the REPL and verify your understanding:

1. `3 + 4 * 2` (should be 11)
2. `(3 + 4) * 2` (should be 14) 
3. `10 - 2 * 3` (should be 4)
4. `5.0 + 3` (should be `8f`)
5. `15 % 3 + 2` (should be 3f)

## Key Takeaways

- Atoms are the building blocks: integers, floats, booleans, characters
- O evaluates expressions **right-to-left**
- Use parentheses to override evaluation order
- Integers and floats mix freely, and a boolean counts as `0` or `1` in arithmetic; characters do not take part
- Integer overflow is an error, not a wrap-around
- Division uses `%` symbol and always returns a float

In the next chapter, we'll learn how to store values in variables and reuse them in calculations.
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

Not yet: `1f` cannot be typed; write `1.0`.

Not yet: `0w` (infinity) and `0n` (not a number) cannot be typed; they only appear as the result of a division.

### Booleans
True or false values:
```
oxidedb> 1b
1b
oxidedb> 0b
0b
```

Not yet: booleans do not take part in arithmetic. `1b + 1` is an error:
```
oxidedb> 1b + 1
Error: Invalid binary operation: Boolean(true) Add Integer(1)
```

### Characters
Single characters enclosed in double quotes (strings such as `"ab"` come in a later chapter):
```
oxidedb> "a"
"a"
oxidedb> "Z"
"Z"
oxidedb> "5"
"5"
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

Dividing by zero is not an error; it follows IEEE 754, so `1 % 0` is `0w` (infinity) and `0 % 0` is `0n` (not a number).

The `/` character is **not** division. After a space it starts a comment, so everything from it to the end of the line is ignored; glued to the previous token it is the *over* adverb, which O does not implement yet:
```
oxidedb> 6 / 2
6
oxidedb> 6/2
Error: adverb '/' not yet implemented
oxidedb> 6 % 2
3f
```
To divide, always use `%`.

### Overflow
Integers are 64-bit. Integer arithmetic that leaves that range is an error, never a silent wrap-around:
```
oxidedb> 9223372036854775807 + 1
Error: Integer overflow
```

The smallest 64-bit value, `-9223372036854775808`, is reserved as the integer null, which prints as `0N`. So that literal reads as a null, and an operation that would land exactly on it is an overflow error:
```
oxidedb> -9223372036854775808
0N
oxidedb> -9223372036854775807 - 1
Error: Integer overflow
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

A minus sign glued to a number, with a space before it, is a negative literal, not subtraction. In q, `2 -1` reads as the two-item list `2 -1`; O has no lists yet, so it rejects the expression. Write `2 - 1` or `2-1` for subtraction:
```
oxidedb> 2 - 1
1
oxidedb> 2-1
1
oxidedb> 2 -1
Error: Unexpected token after expression: Integer(-1)
```

A minus applied to an expression (rather than glued to a number) negates everything to its right, so `-x+3` is `-(x+3)`:
```
oxidedb> x:2
2
oxidedb> -x+3
-5
```

A minus glued to a digit is part of the number, so `-5 + 3` is `-2`; with a space, `- 5 + 3` applies the minus to `5 + 3` and gives `-8`:
```
oxidedb> -5 + 3
-2
oxidedb> - 5 + 3
-8
```

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
- Integers and floats mix freely; booleans and characters do not take part in arithmetic
- Integer overflow is an error, not a wrap-around
- Division uses `%` symbol and always returns a float

In the next chapter, we'll learn how to store values in variables and reuse them in calculations.
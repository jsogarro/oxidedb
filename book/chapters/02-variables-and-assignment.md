# Chapter 2: Variables and Assignment

Variables allow you to store values and reuse them later. In O, like Q, variables are assigned using the colon (`:`) operator.

## Basic Variable Assignment

The syntax for assignment is simple: `variable:value`

```
oxidedb> x:5
5
```

When you assign a value to a variable, O returns the value that was assigned. The variable `x` now contains the value `5`.

## Retrieving Variable Values

To get the value of a variable, simply type its name:

```
oxidedb> x
5
```

## Variable Names

Variable names in O follow these rules:
- Must start with a letter or underscore (O accepts a leading underscore, but q does not, so prefer a letter)
- Can contain letters, numbers, and underscores
- Are case-sensitive

Valid variable names:
```
oxidedb> myVar:10
10
oxidedb> total_count:42
42
oxidedb> x1:100
100
```

A name that starts with a digit is an error: `1x:2` is read as the number `1` followed by a stray name.

## Using Variables in Expressions

Variables can be used in arithmetic expressions just like literal values:

```
oxidedb> y:10
10
oxidedb> z:x+y
15
oxidedb> z
15
```

Remember that expressions are evaluated right-to-left:
```
oxidedb> a:3
3
oxidedb> b:a*2+1
9
oxidedb> b
9
```

This is evaluated as `a * (2 + 1)` = `3 * 3` = `9`

## More Complex Examples

You can use multiple variables together:
```
oxidedb> width:5
5
oxidedb> height:3
3
oxidedb> area:width*height
15
oxidedb> area
15
```

Variables can also store floating-point numbers:
```
oxidedb> pi:3.14159
3.14159
oxidedb> radius:2.5
2.5
oxidedb> circumference:2*pi*radius
15.70795
```

## Variable Reassignment

You can change the value of a variable by assigning to it again:
```
oxidedb> counter:1
1
oxidedb> counter:counter+1
2
oxidedb> counter
2
```

In the expression `counter:counter+1`, the right side (`counter+1`) is evaluated first using the old value of `counter`, then the result is assigned back to `counter`.

## Chained Assignment and Assignment Inside Expressions

Assignment is an expression, and it returns the assigned value. That lets you chain it, and use it in the middle of a larger expression:
```
oxidedb> p:q:7
7
oxidedb> p
7
oxidedb> q
7
oxidedb> (r:1)+2
3
oxidedb> r
1
```

## Variables Persist

In the REPL, variables persist until you exit the session:
```
oxidedb> age:25
25
oxidedb> initial:"A"
"A"
oxidedb> age
25
oxidedb> initial
"A"
```

Strings such as `"Alice"` are not implemented yet. A double-quoted value holds exactly one character, so `name:"Alice"` is an error today. Strings arrive in a later chapter.

## Working with Different Types

Variables can store any atom: integer, float, boolean or character. The type belongs to the value, so the same name can hold a different type later.

```
oxidedb> number:42
42
oxidedb> decimal:3.14
3.14
oxidedb> flag:1b
1b
oxidedb> letter:"X"
"X"
```

O handles type conversions automatically in mixed expressions:
```
oxidedb> int_val:5
5
oxidedb> float_val:2.0
2f
oxidedb> result:int_val*float_val
10f
oxidedb> result
10f
```

## Error Handling

If you try to use a variable that doesn't exist, O will give you an error:
```
oxidedb> undefined_variable
Error: Undefined variable: undefined_variable
```

## Exercises

1. Create variables for your age and the first letter of your name, then display them
2. Calculate the area of a rectangle using width and height variables
3. Create a temperature in Celsius and convert it to Fahrenheit using the formula: `F = 32 + (C * 9) % 5` (remember right-to-left evaluation; the result is a float, e.g. `77f` for 25)
4. Try reassigning a variable and verify the new value

## Key Takeaways

- Use `:` for assignment: `variable:value`
- Variable names are case-sensitive and start with a letter (or underscore)
- Assignment returns its value, so `x:y:7` and `(a:1)+2` work
- Variables can be used in expressions like literal values
- Right-to-left evaluation applies to expressions with variables
- Variables persist throughout the REPL session
- O handles type conversions automatically
- Undefined variables produce errors

In the next chapter, we'll explore vectors - collections of atoms that enable powerful array operations.
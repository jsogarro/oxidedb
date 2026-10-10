# Chapter 3: Vectors and Lists

So far every value has been a single atom. Real data comes in bunches: a week of temperatures, a column of prices, the names of your customers. O is an array language, so a list of values is as easy to write as a single one, and later chapters build tables and queries out of such lists. This chapter is in progress: it covers how to write vectors, store them, compute with them and compare them, and ends with a list of what is still to come.

## Why Vectors?

Here is the whole idea in one line: add 10 to three numbers at once.
```
oxidedb> 1 2 3 + 10
11 12 13
```
No loop, no index: the `+` is applied to every item.

In this book a **list** is the general word for an ordered collection of values, and a **vector** is a list whose items all have the same type. Every list literal in this chapter is a vector. O stores it compactly and applies an operation to every item in one step instead of making you write a loop. A list of numbers is written by putting the numbers next to each other, separated by spaces.

## Vector Literals

### Long vectors
```
oxidedb> 1 2 3
1 2 3
oxidedb> 10 20 30 40
10 20 30 40
```

### Float promotion
If any number in the list is a float, every item becomes a float. The `f` suffix in the output marks a float list in which every item is a whole number, just as it does for a single float:
```
oxidedb> 1 2.5 3
1 2.5 3
oxidedb> 1 2 3f
1 2 3f
oxidedb> 1.5 2
1.5 2
```
`1 2 3f` is the float list `1 2 3`: the `f` on the last number promotes the whole list, not just that item. The `f` suffix is only allowed on the last item, as in q:
```
oxidedb> 1f 2
'parse: invalid literal: 1f...
```

### Boolean vectors
A run of `0` and `1` digits followed by `b` is a boolean vector:
```
oxidedb> 101b
101b
oxidedb> 00b
00b
```
A single `1b` or `0b` is still a boolean atom. Booleans in a list are written glued together as one token: `101b`, not `1b 0b 1b`. Separate booleans (or a boolean and a number) side by side are not a list; see Unfinished Business below. Booleans do take part in arithmetic, as `0` and `1` (see below).
```
oxidedb> 1b
1b
oxidedb> 1b 0b
'nyi: application
```

### Symbols
A symbol is a name that starts with a backtick. It is an atom, a single value, and it is how O will name columns and keys later. A lone backtick is the null symbol. Symbols written back to back make a symbol vector:
```
oxidedb> `apple
`apple
oxidedb> `
`
oxidedb> `a`b`c
`a`b`c
```
Symbol names use letters, digits, `_` and `.`. Write the symbols of a vector with no spaces between the backticks: two symbols with a space between them, `` `a `b ``, are two values side by side, which O reads as application (see Unfinished Business below), not as a vector.

### Strings are lists of characters
A string is a list of characters, so `"abc"` is a character vector. In Chapter 1 we saw that one character in quotes is an atom; two or more make a vector, and empty quotes make the empty character vector:
```
oxidedb> "hello"
"hello"
oxidedb> "a"
"a"
oxidedb> ""
""
oxidedb> "tab\there"
"tab\there"
```
The escapes are the same as for a single character: `\n`, `\t`, `\r`, `\\` and `\"`.

### Atom or list?
The same-looking value can be an atom or a list. `5` is an atom; there is no way to type a one-item list as a plain literal. The display of a one-item list starts with a comma, which is how you will tell them apart once you can build one:

| Written | What it is |
| --- | --- |
| `5` | integer atom |
| `5 6` | integer vector |
| `` `a `` | symbol atom |
| `` `a`b `` | symbol vector |
| `"a"` | character atom |
| `"ab"` | character vector |
| `1b` | boolean atom |
| `10b` | boolean vector |

## The Negative-Literal Trap

A minus sign glued to a digit is part of the number in three places: at the start of a line (`-1 2 3`), after an operator or an opening bracket (`2*-1`), and after a space that follows any value (`2 -1`, `x -1`, `(1) -1`). So after a number `2 -1` is **a list of two numbers**, not subtraction. Subtraction needs spaces on both sides or on neither:
```
oxidedb> 2 -1
2 -1
oxidedb> 2 - 1
1
oxidedb> 2-1
1
oxidedb> 1 -2 3
1 -2 3
oxidedb> -1 2 3
-1 2 3
oxidedb> 2*-1
-2
```
The most common trap is a variable. With `x:5`, `x -1` is **not** subtraction: a name is a value too, so it is two values side by side, and O reports `'nyi: application` (q would apply `x` to `-1`). `x - 1` and `x-1` subtract:
```
oxidedb> x:5
5
oxidedb> x -1
'nyi: application
oxidedb> x - 1
4
oxidedb> x-1
4
```
When in doubt, put spaces around the minus you mean as an operator.

## Storing Vectors in Variables

A vector is a value like any other, so you can assign it to a variable. Assignment returns the vector, and chained assignment gives both names the same value:
```
oxidedb> v:1 2 3
1 2 3
oxidedb> v
1 2 3
oxidedb> a:b:10 20
10 20
oxidedb> b
10 20
oxidedb> name:"Alice"
"Alice"
oxidedb> tags:`red`green
`red`green
```
Reading a variable does not copy the data, so storing a big vector under several names is cheap.

Parentheses group a vector as they group anything else, and a trailing comment is ignored:
```
oxidedb> (1 2 3)
1 2 3
oxidedb> 4 5 6 / a comment
4 5 6
```

## Nulls Inside Vectors

The null literals from Chapter 1 can sit inside a vector. `0N` is the integer null, `0n` the float null and `0w` the float infinity:
```
oxidedb> 1 0N 3
1 0N 3
oxidedb> 1 0n
1 0n
oxidedb> 1 0w
1 0w
```
When the list is promoted to floats, a float vector cannot hold an integer null, so `0N` becomes `0n`:
```
oxidedb> 0N 0n
0n 0n
oxidedb> 0N 1.5
0n 1.5
```

## Arithmetic on Vectors

The four arithmetic operators work on vectors item by item. The rules are the ones from Chapter 1, applied to every item.

### An atom with a vector
The atom is paired with every item, on whichever side it stands:
```
oxidedb> 1 2 3 + 1
2 3 4
oxidedb> 1 + 1 2 3
2 3 4
oxidedb> 10 20 30 * 2
20 40 60
```

### Two vectors
Two vectors of the same length pair up item by item. Vectors of different lengths are an error:
```
oxidedb> 1 2 3 + 4 5 6
5 7 9
oxidedb> 1 2 3 * 4 5 6
4 10 18
oxidedb> 1 2 3 + 4 5
'length
```

### Right to left, as always
Evaluation is still right to left, so `1 2 3 * 2 + 1` is `1 2 3 * (2 + 1)`:
```
oxidedb> 1 2 3 * 2 + 1
3 6 9
```

### Division and promotion
`%` always gives floats. A long vector plus a float promotes to floats, and a boolean vector counts as longs:
```
oxidedb> 1 2 3 % 2
0.5 1 1.5
oxidedb> 1 2 3 + 0.5
1.5 2.5 3.5
oxidedb> 101b + 1
2 1 2
```

### Nulls and overflow
A null stays a null in its own position; overflow in any item is an error for the whole operation:
```
oxidedb> 1 0N 3 + 1
2 0N 4
oxidedb> 1 0n 3 + 1
2 0n 4
oxidedb> 9223372036854775807 1 + 1
'overflow
```

### Symbols and strings
Symbols and characters are not numbers, so arithmetic on them is a `'type` error:
```
oxidedb> `a`b + 1
'type
oxidedb> "abc" + 1
'type
```

### Negating a vector
A minus with a space after it is O's leading minus, applied to everything on its right (q would write `neg 1 2 3`). Glued to the digit it is part of the first number:
```
oxidedb> - 1 2 3
-1 -2 -3
oxidedb> -1 2 3
-1 2 3
```

### The negative-literal trap, with arithmetic
`2 -1 + 1` is the two-item vector `2 -1` plus 1. With a variable the same spelling is application, not subtraction:
```
oxidedb> 2 -1 + 1
3 0
oxidedb> 2 - 1 + 1
0
oxidedb> x:5
5
oxidedb> x -1 + 1
'nyi: application
```

### Variables in arithmetic
```
oxidedb> c:20 25 30
20 25 30
oxidedb> 32 + c * 9 % 5
68 77 86f
```
Read right to left: `9 % 5` is `1.8`, `c * 1.8` is `36 45 54`, and adding `32` gives the Fahrenheit temperatures. The result is a float vector whose items are all whole, so it prints with a trailing `f`.

## Comparing

The comparison verbs ask a question about a pair of values and answer with a boolean: `=` (equal), `<>` (not equal), `<` (less), `<=` (at most), `>` (greater) and `>=` (at least). On two atoms the answer is an atom, `1b` for yes and `0b` for no:
```
oxidedb> 1 < 2
1b
oxidedb> 2 = 3
0b
oxidedb> 2 <> 3
1b
oxidedb> 5 >= 5
1b
```
They work item by item exactly as the arithmetic verbs do, so with a vector the answer is a **boolean vector**, written glued like `101b`. An atom is compared with every item, and two vectors pair up if they have the same length:
```
oxidedb> 1 2 3 = 1 5 3
101b
oxidedb> 10 20 30 > 15
011b
oxidedb> 15 < 10 20 30
011b
oxidedb> 1 2 3 = 1 2
'length
```
The answer is a value like any other, so you can store it:
```
oxidedb> v:1 2 3
1 2 3
oxidedb> m:v>1
011b
oxidedb> m
011b
```
Do not confuse the two jobs of the colon and the equals sign: `x:3` assigns, `x=3` asks whether `x` is 3.

### Still right to left
There is no precedence among the verbs, comparisons included. `1 < 2 + 3` is `1 < (2 + 3)`, which is what you want. The surprise comes when the comparison is on the right of an arithmetic verb: `2 + 3 > 1` is `2 + (3 > 1)`, and a boolean counts as `0` or `1` in arithmetic, so the answer is a number, not a boolean:
```
oxidedb> 1 < 2 + 3
1b
oxidedb> 2 + 3 > 1
3
oxidedb> (2 + 3) > 1
1b
```
Use parentheses to compare the result of arithmetic on the left. As with arithmetic, a minus glued to a digit after a comparison is a negative number:
```
oxidedb> 1 < -2
0b
oxidedb> 1 > -2
1b
```

### Nulls
A null equals a null, and it sorts below every other value, so `0N < 1` is true. This is how q behaves, and it differs from the IEEE rule that a not-a-number never equals anything:
```
oxidedb> 0N = 0N
1b
oxidedb> 0N < 1
1b
oxidedb> 1 < 0N
0b
oxidedb> 1 0N 3 = 1 0N 4
110b
```

### Strings and symbols
Strings are compared character by character, and symbol vectors symbol by symbol:
```
oxidedb> "abc" = "abd"
110b
oxidedb> "abc" = "b"
010b
oxidedb> `a`b = `a`c
10b
oxidedb> `a < `b
1b
```
A character, a symbol and a number are different kinds of value, and comparing across kinds is a `'type` error. This is a deliberate difference from q, which compares a character with a number by its code (`"abc"=1` is `000b` in q):
```
oxidedb> "abc" = 1
'type
oxidedb> `a = 1
'type
```

### Floats are compared exactly
Floats are compared bit for bit, with no tolerance. Most decimal fractions cannot be stored exactly, so a sum that looks like `0.3` can differ from `0.3` in the last place, and `=` reports it. q compares floats with a small tolerance and says yes here, so this is a deliberate difference from q (`(0.1+0.2)=0.3` is `1b` in q):
```
oxidedb> (0.1+0.2) = 0.3
0b
oxidedb> (0.1+0.2) < 0.3
0b
oxidedb> (0.1+0.2) > 0.3
1b
oxidedb> (0.5+0.25) = 0.75
1b
```
Sums of halves and quarters are exact in binary, so that last one is true.

You may be tempted to count how many items match, for example how many entries of `v>1` are true. That needs a way to add up a vector, which does not exist yet, so for now the boolean vector is the answer.

## Take (`#`)

`n # v` takes the first `n` items of `v`. The count goes on the left, the list on the right:
```
oxidedb> 2#1 2 3
1 2
oxidedb> 3#"hello"
"hel"
```
If the count is larger than the list, take wraps around and starts again. A negative count takes from the end, and wraps the same way:
```
oxidedb> 5#1 2
1 2 1 2 1
oxidedb> -2#1 2 3
2 3
oxidedb> -5#1 2 3
2 3 1 2 3
```
Taking nothing keeps the type, and the display of an empty typed vector says which type it is:
```
oxidedb> 0#1 2
`long$()
oxidedb> 0#`a`b
`symbol$()
oxidedb> 0#"ab"
""
```
Taking from an atom repeats it, which is the easy way to make a constant vector. A negative count of an atom gives the same thing, and `-1#7` is a one-item vector (it prints with a leading comma):
```
oxidedb> 3#7
7 7 7
oxidedb> 3#"a"
"aaa"
oxidedb> 3#`a
`a`a`a
oxidedb> -1#7
,7
```
Like every verb, take is right to left, so `2#1 2 3 + 1` is `2#(1 2 3 + 1)`. The count must be a long (a float count is a `'type` error). A list of counts would reshape the data into rows, which O does not do yet, and a count above 10,000,000 items is a `'domain` error (an O limit; q would try to allocate):
```
oxidedb> 2#1 2 3 + 1
2 3
oxidedb> 2.0#1 2 3
'type
oxidedb> 1 2#1 2 3
'nyi: reshape
oxidedb> 1000000000#1
'domain
```

## Join (`,`)

`x , y` puts two lists end to end. An atom counts as a one-item list:
```
oxidedb> 1 2,3 4
1 2 3 4
oxidedb> 1,2 3
1 2 3
oxidedb> 1 2,3
1 2 3
oxidedb> "ab","c"
"abc"
oxidedb> `a`b,`c
`a`b`c
```
An empty list on either side disappears:
```
oxidedb> 1 2,0#1 2
1 2
oxidedb> (0#1 2),1 2
1 2
```
Join **never converts types**. When the two sides have the same type the result is a vector, but joining a long with a float gives a *general list*, a list whose items keep their own types, not a float vector. A general list is displayed one item per line (O's display of nested lists will be refined later), and you cannot yet type one directly:
```
oxidedb> 1,2.5
1
2.5
oxidedb> 1 2,3.0
1
2
3f
oxidedb> 1,"a"
1
"a"
```
This is the same in q. Compare this with the literal `1 2.5`, which the reader promotes to a float vector before join is ever involved. And because `,` is just another verb, the right-to-left rule applies: `1 2,3 = 3` is `1 2,(3 = 3)`, a long vector joined to a boolean, so it is a general list too, while `(1 2,3) = 1 2 4` compares the joined vector:
```
oxidedb> 1 2,3 = 3
1
2
1b
oxidedb> (1 2,3) = 1 2 4
110b
```

## Unfinished Business

Three things that look like they should work are not supported yet. Each gives a clear error rather than a wrong answer.

Two values side by side that do not form a literal vector are, in q, the left one applied to the right one: indexing a list or calling a function. That means any two nouns, of the same kind or not: `1 "a"`, `` `a `b ``, `"ab" "cd"`, `(1 2) 3`, `x 1 2` and `1b 0b`. It arrives with indexing:
```
oxidedb> 1 "a"
'nyi: application
oxidedb> `a `b
'nyi: application
oxidedb> (1 2) 3
'nyi: application
oxidedb> x 1 2
'nyi: application
```
General lists, written with parentheses and semicolons, are not built yet:
```
oxidedb> (1;2;3)
'nyi: general lists
```
Adverbs such as over are also still to come:
```
oxidedb> 1 2 3/2
'nyi: adverb '/'
```

## Coming Next

These parts of the chapter will be added as the features arrive. None of them work yet, so there are no samples for them:

- `til` and `count`
- indexing a vector
- general (mixed or nested) lists, written with parentheses and semicolons
- assigning to an item of a vector

## Exercises

1. Write the vector holding 3, 1, 4, 1, 5 and store it in `pi_digits`.
2. Write a float vector of three whole numbers (hint: one item needs a decimal point or an `f`) and check that every item shows as a float.
3. What does `7 -3` print? Which two spellings give you `4`?
4. Store a string in a variable and print it back. Is `"x"` a string or a character?
5. Write a symbol vector of three names. Why can there be no spaces between the backticks?
6. Predict the output of `5 0N 2.5`, then run it.
7. A shop sells three items at prices `p:10 20 30` in quantities `q:1 2 3`. Write the expression for the cost of each line (price times quantity), then the cost of each line with a flat fee of 5 added.
8. Predict `10 20 30 - 1 2 3 * 2`, then `(10 20 30 - 1 2 3) * 2`. Which one subtracts first?
9. Predict `5 3 8 > 4`, then store the answer in `big`. What does `3 > 2 + 5` print, and why is `3 + 2 > 5` not a boolean?

## Key Takeaways

- Numbers next to each other form a vector; one float makes the whole vector float
- `101b` is a boolean vector, written glued
- `` `a `` is a symbol atom and `` `a`b `` a symbol vector
- A string is a character vector; one character in quotes is an atom
- `2 -1` is a two-item vector; write `2 - 1` for subtraction
- Two values side by side that are not one literal are application, which is not built yet (`x -1` is the classic surprise)
- Variables hold vectors, and reading one does not copy it
- A null inside a float vector is `0n`
- `= <> < <= > >=` compare item by item and give booleans (`1b`, or a boolean vector like `101b`); null equals null and sorts lowest; comparing across kinds is `'type`; `=` asks, `:` assigns
- `+ - * %` work item by item on vectors: an atom is paired with every item, two vectors pair up if they have the same length (`'length` otherwise), `%` gives floats, and symbols and strings are `'type`
- `n#v` takes the first `n` items (wrapping; negative from the end; an atom repeats); `x,y` joins and never converts types, so `1,2.5` is a general list; `!` and reshape are still to come
- Indexing, `til`, `count` and the rest are still to come

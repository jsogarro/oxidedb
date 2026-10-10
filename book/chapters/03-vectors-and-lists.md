# Chapter 3: Vectors and Lists

So far every value has been a single atom. Real data comes in bunches: a week of temperatures, a column of prices, the names of your customers. O is an array language, so a list of values is as easy to write as a single one, and later chapters build tables and queries out of such lists. This chapter is in progress: it covers how to write vectors and store them, and ends with a list of what is still to come.

## Why Vectors?

A **vector** is a list whose items all have the same type. O stores it compactly and, once vector arithmetic arrives, will apply an operation to every item in one step instead of making you write a loop. A list of numbers is written by putting the numbers next to each other, separated by spaces.

## Vector Literals

### Long (integer) vectors
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
`1 2 3f` is the float list `1 2 3`: the `f` on the last number promotes the whole list, not just that item.

### Boolean vectors
A run of `0` and `1` digits followed by `b` is a boolean vector:
```
oxidedb> 101b
101b
oxidedb> 00b
00b
```
A single `1b` or `0b` is still a boolean atom. Booleans do not mix with numbers in a list: `1 1b` is a `'type` error, and so is `1b 0b` (use `10b`).
```
oxidedb> 1b
1b
oxidedb> 1 1b
'type
oxidedb> 1b 0b
'type
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
Symbol names use letters, digits, `_` and `.`. Write the symbols of a vector with no spaces between them.

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

A minus sign glued to a digit is part of the number when it follows a space, so after a number `2 -1` is **a list of two numbers**, not subtraction. Subtraction needs spaces on both sides or on neither:
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
When the list is promoted to floats, an integer null becomes the float null `0n`, because only floats can hold a float null:
```
oxidedb> 0N 0n
0n 0n
oxidedb> 0N 1.5
0n 1.5
```

## Unfinished Business

Two things that look like lists are not supported yet, and both give a clear error rather than a wrong answer. Putting values of different kinds side by side, such as `1 "a"`, would be a function applied to an argument in q, and O has no functions yet:
```
oxidedb> 1 "a"
'nyi: application
oxidedb> x 1 2
'nyi: application
```
Arithmetic on a vector is not built yet either; for now a vector can be stored and displayed but not computed with:
```
oxidedb> 1 2 3 + 1
'nyi: vector arithmetic
```
Adverbs such as over are also still to come:
```
oxidedb> 1 2 3/2
'nyi: adverb '/'
```

## Coming Next

These parts of the chapter will be added as the features arrive. None of them work yet, so there are no samples for them:

- arithmetic on vectors (an atom with a vector, and two vectors item by item)
- `til` and `count`
- indexing a vector
- take (`#`) and join (`,`)
- comparisons that return boolean vectors
- general (mixed or nested) lists, written with parentheses and semicolons
- assigning to an item of a vector

## Exercises

1. Write the vector holding 3, 1, 4, 1, 5 and store it in `pi_digits`.
2. Write a float vector of three whole numbers (hint: one item needs a decimal point or an `f`) and check that every item shows as a float.
3. What does `7 -3` print? Which two spellings give you `4`?
4. Store a string in a variable and print it back. Is `"x"` a string or a character?
5. Write a symbol vector of three names. Why can there be no spaces between the backticks?
6. Predict the output of `5 0N 2.5`, then run it.

## Key Takeaways

- Numbers next to each other form a vector; one float makes the whole vector float
- `101b` is a boolean vector; booleans do not mix with numbers
- `` `a `` is a symbol atom and `` `a`b `` a symbol vector
- A string is a character vector; one character in quotes is an atom
- `2 -1` is a two-item vector; write `2 - 1` for subtraction
- Variables hold vectors, and reading one does not copy it
- A null inside a float vector is `0n`
- Vector arithmetic, indexing and the rest are still to come

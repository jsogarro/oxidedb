# Chapter 3: Vectors and Lists

So far every value has been a single atom. Real data comes in bunches: a week of temperatures, a column of prices, the names of your customers. O is an array language, so a list of values is as easy to write as a single one, and later chapters build tables and queries out of such lists. This chapter is in progress: it covers how to write vectors, store them, compute with them, compare them and pick items out of them, and ends with a list of what is still to come.

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
A single `1b` or `0b` is still a boolean atom. Booleans in a list are written glued together as one token: `101b`, not `1b 0b 1b`. Separate booleans (or a boolean and a number) side by side are not a list; they are two values side by side, which O reads as application (see Indexing below). Booleans do take part in arithmetic, as `0` and `1` (see below).
```
oxidedb> 1b
1b
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
Symbol names use letters, digits, `_` and `.`. Write the symbols of a vector with no spaces between the backticks: two symbols with a space between them, `` `a `b ``, are two values side by side, which O reads as application (see Indexing below), not as a vector.

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
The most common trap is a variable. With `x:1 2 3`, `x -1` is **not** subtraction: a name is a value too, so `x -1` is two values side by side, and a value followed by a value is application. Here that means indexing `x` with `-1`, which is out of range, so the answer is a null (Indexing below explains why). `x - 1` and `x-1` subtract:
```
oxidedb> x:1 2 3
1 2 3
oxidedb> x -1
0N
oxidedb> x - 1
0 1 2
oxidedb> x-1
0 1 2
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
A minus with a space after it is O's leading minus, applied to everything on its right. Glued to the digit it is part of the first number:
```
oxidedb> - 1 2 3
-1 -2 -3
oxidedb> -1 2 3
-1 2 3
```
q has no leading minus on an expression. It negates with the keyword `neg`, and so does O: `neg` is a function, so it takes everything to its right in the same way (see `til` and `count` below for how a name is applied). It works on atoms, booleans and vectors, and a null stays a null. (`neg 1b` is `-1i` in q, an int; O has no int type, so its result is the long `-1`.)
```
oxidedb> neg 5
-5
oxidedb> neg 1 2 3
-1 -2 -3
oxidedb> neg 1b
-1
oxidedb> neg 0N
0N
oxidedb> 1 + neg 2
-1
oxidedb> neg 1 + 2
-3
```

### The negative-literal trap, with arithmetic
`2 -1 + 1` is the two-item vector `2 -1` plus 1. With a variable the same spelling is application, not subtraction, and application takes everything to its right, so `x -1 + 1` is `x (-1 + 1)`, which is `x 0`:
```
oxidedb> 2 -1 + 1
3 0
oxidedb> 2 - 1 + 1
0
oxidedb> x -1 + 1
1
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

## `til` and `count`

Two named functions come with O. `til n` makes the vector `0 1 ... n-1`, and `count x` is the number of items in `x`. You apply a name by writing it before its argument:
```
oxidedb> til 5
0 1 2 3 4
oxidedb> count 1 2 3
3
oxidedb> count "hello"
5
oxidedb> count 7
1
```
A function takes **everything to its right** as its argument, which is the right-to-left rule again. `til 3+2` is `til 5`, not `(til 3) + 2`, and functions chain without parentheses:
```
oxidedb> til 3+2
0 1 2 3 4
oxidedb> (til 3) + 2
2 3 4
oxidedb> count til 5
5
oxidedb> count til 3 + 2
5
oxidedb> 10 * til 3
0 10 20
```
The argument can also go in square brackets, `til[5]`, which is the same thing written the way you write a call with several arguments. A function that wants one argument is a `'rank` error with two or none. `til` of a negative number or the null is a `'domain` error, and of anything but a long a `'type` error:
```
oxidedb> til[5]
0 1 2 3 4
oxidedb> til[1;2]
'rank
oxidedb> til -1
'domain
oxidedb> til 2.5
'type
oxidedb> til[]
'rank
```
(In q, `til 1b` is `,0` and `til[]` passes a null argument; O reports `'type` and `'rank`.) A name like `til` belongs to the language, so you cannot assign to it, and on its own it is not yet a value you can print:
```
oxidedb> til:3
'parse: cannot assign to builtin til
oxidedb> til
'nyi: til as a value
```

## Indexing

A vector is applied to an index to get items out of it. Counting starts at 0. You can write the index in brackets or put it after the name; a single index gives an item, and a vector of indexes gives a vector of items:
```
oxidedb> v:10 20 30
10 20 30
oxidedb> v[1]
20
oxidedb> v 1
20
oxidedb> v[0 2]
10 30
oxidedb> v 0 2
10 30
oxidedb> v[til 3]
10 20 30
oxidedb> i:2
2
oxidedb> v i
30
```
The result has one item for each index, in the order you asked, and indexes can repeat: `v[2 1 0 0]` is `30 20 10 10`. Strings and symbol vectors index the same way:
```
oxidedb> "abc" 1
"b"
oxidedb> "abc" 0 2
"ac"
oxidedb> `a`b`c 2
`c
```

### Out of range is a null
An index past the end gives the null of the vector's type, not an error. A negative index is also out of range; O does not count from the end:
```
oxidedb> v[5]
0N
oxidedb> v[3]
0N
oxidedb> v[-1]
0N
oxidedb> v[1 5]
20 0N
oxidedb> `a`b[5]
`
oxidedb> "abc"[5]
" "
oxidedb> 1.5 2.5[5]
0n
oxidedb> 101b[7]
0b
```
Each type has its own null: `0N` for longs, `0n` for floats, the null symbol, a blank for characters and `0b` for booleans (there is no boolean null).

### Brackets, juxtaposition and right to left
Square brackets bind tighter than anything else, so `v[0] + 1` is `(v[0]) + 1`, while `v 0 + 1` is `v (0 + 1)`, because a name written before a value applies to everything on its right. Brackets can be chained, and what a bracket returns can be indexed again:
```
oxidedb> v[0] + 1
11
oxidedb> v 0 + 1
20
oxidedb> v[0 2][1]
30
oxidedb> v[1 > 10]
10
```
The last line is `v[0b]`: a boolean index counts as 0 or 1, so a comparison on the right of a name is easy to misread. When the index is an expression, put it in brackets.

The argument is evaluated before the vector it indexes, as with every right-to-left rule, so an assignment inside the brackets is done first:
```
oxidedb> v[j:1]
20
oxidedb> j
1
```

### What cannot be indexed
The index must be a long or a boolean (or a vector of them); a float, symbol or character index is a `'type` error. An atom cannot be indexed either: `x 1` with `x:5` is a `'type` error (q treats a number applied to an argument as a file handle, which O does not imitate). A single index is all a vector takes; more is a `'rank` error (q says `'type`). Empty brackets give the vector back:
```
oxidedb> v[1.0]
'type
oxidedb> x:5
5
oxidedb> x 1
'type
oxidedb> v[0;1]
'rank
oxidedb> v[]
10 20 30
```

## Changing items

An index on the left of a colon replaces those items. The statement's value is what the index reads afterwards, so `5` here, and the vector itself has changed:
```
oxidedb> d:10 20 30
10 20 30
oxidedb> d[0]:5
5
oxidedb> d
5 20 30
```
Several items change at once with a vector of indexes. A vector of values goes in pairwise, and a single value is used for every index:
```
oxidedb> d[0 2]:7 8
7 8
oxidedb> d
7 20 8
oxidedb> d[1 2]:0
0 0
oxidedb> d
7 0 0
```
If an index is repeated, the last value wins, and the statement's value is read back from the vector, so it shows the winner twice:
```
oxidedb> d[0 0]:1 2
2 2
oxidedb> d
2 0 0
```
The copy stays unchanged. Giving a vector a second name does not copy it until one of the two is changed, and then only the one you changed is different:
```
oxidedb> e:d
2 0 0
oxidedb> d[0]:99
99
oxidedb> d
99 0 0
oxidedb> e
2 0 0
```
An assignment is an expression like any other and takes everything on its right, so it can sit inside a bigger one. The value is evaluated first and then the index, right to left:
```
oxidedb> 1+d[0 1]:5 6
6 7
oxidedb> d
5 6 0
```
Strings are vectors of characters, so they change in the same way:
```
oxidedb> t:"abc"
"abc"
oxidedb> t[0]:"x"
"x"
oxidedb> t
"xbc"
```

### The type must match

A vector has one type, and the new item must have exactly that type. O never converts it for you: a float does not fit in a long vector, a long does not fit in a float vector, and a boolean is not a long. Any of these is a `'type` error, and the vector is left as it was:
```
oxidedb> d[0]:1.5
'type
oxidedb> d[1]:`a
'type
oxidedb> f:1.5 2.5
1.5 2.5
oxidedb> f[0]:1
'type
oxidedb> d
5 6 0
```
A null of the right type is fine (`d[0]:0N`). Only a general list takes items of any type.

### Out of range is an error

Reading past the end gives a null, but assigning past the end is an error, because the vector cannot grow this way. The error is `'length` (not `'index`; q has no such error). A negative index and the null `0N` are out of range too. If any index is out of range nothing is changed. The count of values must match the count of indexes as well:
```
oxidedb> d[5]
0N
oxidedb> d[5]:1
'length
oxidedb> d[-1]:1
'length
oxidedb> d[0 1]:1 2 3
'length
oxidedb> d
5 6 0
```
So a read is forgiving and a write is strict. To make a vector longer, join to it (`d,7`) and bind the result.

### What can be assigned to

Only a name can have its items changed. The name must hold a vector or a list: an atom is a `'type` error, and a name with no value is a `'length` error (q treats a name it does not know as an empty list, so every index is out of range). Changing an item of an item (`d[0][1]:5`), several indexes in one bracket and the combined forms such as `d[0]+:1` are not supported yet:
```
oxidedb> y:5
5
oxidedb> y[0]:1
'type
oxidedb> nothing[0]:1
'length
oxidedb> d[0][1]:2
'nyi: depth assignment
oxidedb> d[0]+:1
'nyi: compound assignment
```

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

Some things that look like they should work are not supported yet. Each gives a clear error rather than a wrong answer.

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
Changing an item of an item, and the combined forms of assignment, are the last gaps in changing items (see above).

## Coming Next

These parts of the chapter will be added as the features arrive. None of them work yet, so there are no samples for them:

- general (mixed or nested) lists, written with parentheses and semicolons
- changing an item of an item (`d[0][1]:5`) and the combined forms (`d[0]+:1`)
- several statements on one line, separated by `;`
- adverbs and functions

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
10. Predict `til 3 + 2` and `(til 3) + 2`, then run both. Which one adds first?
11. With `v:10 20 30 40 50`, write an expression for its last item without typing `4` (hint: `count`, and remember that `v count v - 1` does not do what it seems to).
12. What is `v[1 7 -1]`? Why is none of the three an error?
13. With `x:1 2 3`, predict `x -1`, `x - 1` and `x -1 + 1`, then run them.
14. Predict `-3#1 2` and `4#`b`c`, then run them. What does `0#1 2.5` print, and why is that not an error?
15. Predict `2,3#4 5` (which verb runs first?), then run it.
16. Make the vector `7 7 7 7` without typing four sevens, and a six-item vector `1 2 1 2 1 2` from `1 2`.
17. Predict `(0.2+0.1) = 0.3` and `(0.5+0.5) = 1.0`. Which one is true, and why?
18. Is `(1,2) , 3.5` a float vector? Predict, then run it.
19. With `a:1 2 3 4 5`, change the middle item to 0 and the first and last to 9 in two statements. Print `a`.
20. Predict `b:a`, then `a[0]:7`: what are `a` and `b`? Predict `a[5]:1`, `a[0]:1.5` and `a[5]` before you run them, and say which two are errors and why the third is not.

## Key Takeaways

- Numbers next to each other form a vector; one float makes the whole vector float
- `101b` is a boolean vector, written glued
- `` `a `` is a symbol atom and `` `a`b `` a symbol vector
- A string is a character vector; one character in quotes is an atom
- `2 -1` is a two-item vector; write `2 - 1` for subtraction
- Two values side by side that are not one literal are application: a vector applied to an index, a name applied to its argument (`x -1` with a vector `x` is the classic surprise: it indexes with -1 and gives a null)
- Variables hold vectors, and reading one does not copy it
- A null inside a float vector is `0n`
- `= <> < <= > >=` compare item by item and give booleans (`1b`, or a boolean vector like `101b`); null equals null and sorts lowest; comparing across kinds is `'type`; `=` asks, `:` assigns
- `til n` makes `0 .. n-1` and `count` counts items; a name is applied to everything on its right (`til 3+2` is `til 5`), or to its brackets (`til[5]`)
- `v[i]` and `v i` index a vector: an atom for an atom index, a vector for a vector of indexes; out of range or negative gives the type's null, never an error; an atom cannot be indexed, and too many indexes are `'rank`
- `neg` negates, as O's leading minus does
- `+ - * %` work item by item on vectors: an atom is paired with every item, two vectors pair up if they have the same length (`'length` otherwise), `%` gives floats, and symbols and strings are `'type`
- `n#v` takes the first `n` items (wrapping; negative from the end; an atom repeats); `x,y` joins and never converts types, so `1,2.5` is a general list
- `v[i]:x` replaces items in place (`v[0 2]:7 8` pairwise, `v[0 2]:9` for every index, the last of a repeated index wins) and a copy made with another name is not changed; the value must have the vector's exact type (`'type`, never converted) and an out-of-range index is `'length` where a read gives a null
- General lists, changing an item of an item, `;` statements, adverbs and functions are still to come

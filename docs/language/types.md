# Types

Adamantium supports signed integers, unsigned integers, floating-point values, strings, booleans, `None`, enums, classes, lists, optional values, and offsets.

```adamantium
var count = 10:int;
var small = 10:i16;
var ratio = 1.5:f64;
var text = "hello":string;
```

Defaults are `i32` for integer literals and `f64` for decimal literals. Explicit conversions use `value.as(Type)` and are checked by the compiler.

## Strings

Strings are immutable UTF-8 values. Assignment may share their backing bytes because string contents cannot be changed. Concatenation with `+` creates a new runtime-owned string. Comparisons use string contents and support `==`, `!=`, `<`, `<=`, `>` and `>=`.

```adamantium
var greeting = "Hello, " + "Adamantium";
var count = greeting.length;
var same_count = greeting.length();
var first = greeting[0];
```

`length` counts Unicode scalar values rather than UTF-8 bytes. Indexing uses the same character positions and returns one character as a string. Strings cannot be changed through an index. An out-of-bounds index produces a recoverable runtime error and can be handled by `try`.

## Lists

`List` stores homogeneous values. The compiler can infer the element type from
a non-empty literal, or the type can be written explicitly.

```adamantium
var values = List[1, 2, 3];
var empty = List[]:List[int];
var nested = List[List[1, 2], List[3, 4]];

values[1] = 9;
nested[0][1] = 8;
```

At runtime, a List value is represented by two machine words: a pointer to a
contiguous allocation and its element count. Every element occupies the common
16-byte runtime `Value` representation. List assignment creates an independent
copy, including recursive copies of nested Lists. Iteration visits elements in
index order. Reading or writing an index outside `0..length` stops normal
execution with a runtime error that reports the index and length.

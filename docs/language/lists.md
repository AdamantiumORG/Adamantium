# Lists

Adamantium Lists are homogeneous collections. A non-empty literal infers its
element type. Empty Lists need an explicit type.

```adamantium
var values = List[1, 2, 3];
var empty = List[]:List[int];
var nested = List[List[1, 2], List[3, 4]];
```

Use a zero-based integer index to read or replace an element. Index targets can
be nested.

```adamantium
print.newline(values[0]);
values[1] = 9;
nested[0][1] = 8;
```

`for value in values { ... }` visits each element in index order.

## Copy semantics

Assigning a List creates an independent allocation. Nested Lists are copied
recursively, so changing any level of the copy does not change the source.
Class elements follow the language's class value-copy rules. Immutable string
storage may be shared.

## Runtime representation

A List value occupies two machine words: a pointer to a contiguous allocation
and the element count. Every element uses the runtime's 16-byte `Value`
representation. The NASM backend emits allocation, element loads and stores,
recursive copy loops, ordered iteration, and bounds checks.

Every read and write checks `index < length`. An invalid index ends normal
execution with exit code 2 and reports both the invalid index and List length.

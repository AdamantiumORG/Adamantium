# Types

Adamantium supports signed integers, unsigned integers, floating-point values, strings, booleans, `None`, enums, classes, lists, optional values, and offsets.

```adamantium
var count = 10:int;
var small = 10:i16;
var ratio = 1.5:f64;
var text = "hello":string;
```

Defaults are `i32` for integer literals and `f64` for decimal literals. Explicit conversions use `value.as(Type)` and are checked by the compiler.

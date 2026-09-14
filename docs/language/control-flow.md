# Control flow

Adamantium supports `if`, `match`, `for`, `while`, `until`, `loop`, `break`, `continue`, `return`, and `exit`.

## Process exit

`exit()` stops the program immediately and returns code `0` to the operating system:

```adamantium
exit();
```

Pass a named `code` argument to return an explicit status:

```adamantium
exit(code=23);
```

Exit codes use `u8` and must be in the portable range `0..255`. Other value types and integer literals outside this range are compile-time errors. A variable passed as the code must therefore have type `u8`.

# Errors and diagnostics

Compiler diagnostics contain an error or warning code, source location, highlighted line, context, and a suggestion when available.

Runtime code may use:

```adamantium
warn("operation continued");
panic("operation failed");
```

`warn` reports the source line and continues. `panic` reports the source line and stops unless it is handled by `try`. A `try` expression returns `None` on success or an optional error string on failure.

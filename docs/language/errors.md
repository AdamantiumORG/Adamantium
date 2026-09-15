# Errors and diagnostics

Compiler diagnostics contain an error or warning code, source location, highlighted line, context, and a suggestion when available.

Runtime code may use:

```adamantium
warn("operation continued");
panic("operation failed");
```

`warn` reports the source line and continues. `panic` reports the source line and stops unless it is handled by `try`. A `try` expression returns `None` on success or an optional error string on failure.

The runtime stores failures as structured `RuntimeError` values containing a
kind, stable code, message, and optional source line. Recoverable failures use
kind `Recoverable` and codes beginning with `R`. Explicit `panic` uses kind
`Panic` and codes beginning with `P`. Package failures have kind `Package`.
This distinction is preserved while `try` is active, although the current
language-level result remains an optional formatted string.

Known source locations are retained for explicit panics, List bounds failures,
and invalid optional-value access. Operations whose AST does not retain an
operator position report the error without inventing a line number.

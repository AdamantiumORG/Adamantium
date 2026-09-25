# Adamantium frontend fuzzing

Install `cargo-fuzz`, then run:

```text
cargo install cargo-fuzz
cargo fuzz run lexer
cargo fuzz run frontend
```

The `lexer` target sends arbitrary valid UTF-8 through the recovering lexer.
It may produce tokens and lexical errors, but it must never panic. The
`frontend` target continues through generic expansion, module parsing, name
resolution and Professional Mode validation. Every crash is a compiler bug.
Add minimized crashes to the language regression suite before fixing them.

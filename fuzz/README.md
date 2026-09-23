# Adamantium frontend fuzzing

Install `cargo-fuzz`, then run:

```text
cargo install cargo-fuzz
cargo fuzz run frontend
```

The target sends arbitrary UTF-8 source code through lexing, generic expansion,
module parsing, name resolution and Professional Mode validation. Every crash
is a compiler bug. Add minimized crashes to the language regression suite
before fixing them.

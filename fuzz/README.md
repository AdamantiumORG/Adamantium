# Adamantium frontend fuzzing

Install `cargo-fuzz`, then run:

```text
cargo install cargo-fuzz
cargo fuzz run lexer
cargo fuzz run frontend
cargo fuzz run pipeline
cargo fuzz run codegen
cargo fuzz run runtime
cargo fuzz run package_loader
```

The `lexer` target sends arbitrary valid UTF-8 through the recovering lexer.
It may produce tokens and lexical errors, but it must never panic. The
`frontend` target continues through generic expansion, module parsing, name
resolution and Professional Mode validation. Every crash is a compiler bug.
Add minimized crashes to the language regression suite before fixing them.

`codegen` generates bounded IR instruction streams for the NASM and both LLVM
ARM64 emitters. `runtime` exercises the safe value and type boundary, while
`package_loader` sends arbitrary bytes through WASM validation and export
discovery. CI runs every target with a fixed time budget; scheduled runs provide
longer coverage without making ordinary pull requests unbounded.

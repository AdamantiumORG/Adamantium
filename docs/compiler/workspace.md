# Cargo workspace

Adamantium uses a Cargo workspace with one crate for each compiler responsibility. The current production pipeline lives in `adamantium-cli` while its modules are moved behind the public crate boundaries incrementally.

## Crates

| Crate | Responsibility |
| --- | --- |
| `adamantium-cli` | Commands and the `adamantium` executable |
| `adamantium-compiler` | Compiler pipeline orchestration |
| `adamantium-ast` | Shared syntax tree data |
| `adamantium-lexer` | Source tokenization |
| `adamantium-parser` | Token parsing |
| `adamantium-types` | Type representation and inference |
| `adamantium-semantics` | Name and semantic analysis |
| `adamantium-diagnostics` | Errors and warnings |
| `adamantium-ir` | Compiler intermediate representation |
| `adamantium-codegen` | IR to assembly generation |
| `adamantium-nasm` | NASM command construction |
| `adamantium-linker` | Native linker command construction |
| `adamantium-runtime` | Native runtime linked into programs |
| `adamantium-project` | Project layout and manifests |
| `adamantium-packages` | Package resolution and installation |
| `adamantium-wasm` | Package WASM validation and ABI |
| `adamantium-testing` | Language testing and affected-crate selection |

Every crate has tests in its own `tests/` directory. Run one crate with:

```text
cargo test -p adamantium-parser
```

Run the complete workspace with:

```text
cargo test --workspace --all-targets
```

## Selective CI

`affected-crates` reads Cargo metadata, maps changed files to workspace packages, and follows reverse dependencies transitively. CI creates a separate operating-system matrix job for every returned package. A root-level change selects the whole workspace because it can affect every crate.

For example, inspect the jobs selected by a lexer change with:

```text
cargo run -p adamantium-testing --bin affected-crates -- --changed crates/adamantium-lexer/src/lib.rs
```

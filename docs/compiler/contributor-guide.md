# Compiler contributor getting started

Run commands from the repository root. A frontend-only change needs Rust; native
program tests additionally need NASM and a supported linker.

```text
cargo check --locked --workspace --all-targets
cargo test --locked -p adamantium-lexer -p adamantium-parser
cargo run -p adamantium-cli -- check example-project
```

## Follow a source file through the compiler

1. `adamantium-project` loads `project.toml`, requirements, and `code/main.ad`.
2. `adamantium-lexer` scans UTF-8 into token kinds and byte spans.
3. `adamantium-parser` builds semantic AST nodes and recovers at statement and
   block boundaries after syntax errors.
4. `adamantium-semantics` resolves names and checks access rules.
5. `adamantium-types` provides canonical types and stable type identifiers.
6. HIR and MIR make resolved definitions and control flow explicit.
7. `adamantium-ir` provides the target-independent backend contract.
8. Codegen emits NASM for x86-64 or LLVM IR for the developing ARM64 path.
9. `adamantium-nasm`, `adamantium-linker`, and `adamantium-runtime` produce and
   support the native executable.

The detailed invariants between these steps are in [phase contracts](phases.md).

## Choose the right crate

| Change | Start in | Primary verification |
| --- | --- | --- |
| Token, keyword, literal, comment | `adamantium-lexer` | `cargo test -p adamantium-lexer` |
| Expression or grammar | `adamantium-parser` and CLI syntax parser | parser tests plus language regressions |
| AST shape or source span | `adamantium-ast` | AST and lowering span tests |
| Name access, imports, visibility | `adamantium-semantics` | semantic and diagnostics tests |
| Type or conversion | `adamantium-types` and CLI checker | type tests plus valid/invalid programs |
| Optimization | CLI optimizer and typed IR | optimizer tests and performance workflow |
| Native instruction generation | CLI codegen or `adamantium-codegen` | codegen tests and native suite |
| Package format or WASM validation | `adamantium-packages`, `adamantium-wasm` | package tests and package-loader fuzzing |
| Runtime behavior | `adamantium-runtime` | runtime and ignored native tests |
| CLI command or project layout | `adamantium-cli`, `adamantium-project` | CLI integration scripts |

Do not make a backend inspect parser tokens or AST details. Add a semantic field
to typed IR instead. Preserve the original `Span` whenever a source construct is
lowered, and use the shared diagnostic model rather than formatting ad hoc
errors inside a compiler phase.

## Add a regression

Small crate contracts belong in that crate's unit or integration tests. User
visible language behavior also belongs under `tests/valid` or `tests/invalid`.
Native runtime behavior belongs in the platform native suites. Malformed input
that once caused a crash should become a deterministic test even when it was
first found by fuzzing.

Before opening a pull request, run the checks listed in the repository
[contribution guide](../../CONTRIBUTING.md). CI selects changed crates and their
downstream dependents, then runs the complete platform suite before publishing
any Nightly artifact.

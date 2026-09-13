# Compiler architecture

Adamantium currently uses one Rust binary crate with a separately built static runtime. The compilation path is:

```text
CLI and project loading
        |
lexer and parser
        |
name resolution and AST validation
        |
type checking and typed instructions
        |
x86-64 NASM generation
        |
NASM and platform linker
        |
native executable
```

The current modules are intentionally kept in one crate while their boundaries are stabilized. `syntax` currently owns lexing, parsing, AST declarations, module combination, visibility, imports, and part of name resolution. `typed` validates types and produces typed instructions. `codegen` translates those instructions to NASM. `diagnostics`, `packages`, and `language_tests` provide supporting services.

## Planned workspace

The long-term workspace may contain focused crates such as `adamantium-cli`, `adamantium-lexer`, `adamantium-parser`, `adamantium-ast`, `adamantium-semantics`, `adamantium-types`, `adamantium-diagnostics`, `adamantium-codegen`, `adamantium-nasm`, `adamantium-linker`, `adamantium-runtime`, `adamantium-project`, `adamantium-packages`, `adamantium-wasm`, and `adamantium-testing`.

Crates should be extracted only after the corresponding in-crate API has a clear input, output, ownership model, and diagnostic boundary. This avoids cyclic dependencies and prevents internal data structures from becoming public APIs accidentally.

Recommended extraction order:

1. AST and shared source locations.
2. Lexer and parser.
3. Diagnostics and type representation.
4. Semantic analysis.
5. Code generation, NASM invocation, and linking.
6. Project, package, WASM, and testing services.
7. Formatter, standard library, and LSP after their contracts exist.

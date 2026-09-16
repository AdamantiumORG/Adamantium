# Compiler architecture

Adamantium is organized as a Rust workspace with a CLI and focused compiler crates. The current compilation path is:

```text
CLI and project loading
        |
lexer and parser
        |
name resolution and AST validation
        |
type checking and target-independent typed IR
        |
x86-64 NASM generation
        |
NASM and platform linker
        |
native executable
```

`adamantium-cli` currently owns the mature parser, semantic checker, and NASM generator while their public crate APIs are stabilized. The checker lowers parsed syntax into `adamantium_ir::typed::Program<Type, Value>`. `adamantium-ir` owns the target-independent control-flow, expression, class, package-call, and operator representation. It has no dependency on the parser or a native backend.

The existing NASM generator consumes this shared typed IR. A future LLVM ARM64 generator can consume the same IR without translating parser syntax or depending on x86-64 register conventions.

## Workspace direction

The workspace contains focused crates including `adamantium-cli`, `adamantium-lexer`, `adamantium-parser`, `adamantium-ast`, `adamantium-semantics`, `adamantium-types`, `adamantium-diagnostics`, `adamantium-ir`, `adamantium-codegen`, `adamantium-nasm`, `adamantium-linker`, `adamantium-runtime`, `adamantium-project`, `adamantium-packages`, `adamantium-wasm`, and `adamantium-testing`.

Crates should be extracted only after the corresponding in-crate API has a clear input, output, ownership model, and diagnostic boundary. This avoids cyclic dependencies and prevents internal data structures from becoming public APIs accidentally.

Recommended extraction order:

1. AST and shared source locations.
2. Lexer and parser.
3. Diagnostics and type representation.
4. Semantic analysis.
5. Code generation, NASM invocation, and linking.
6. Project, package, WASM, and testing services.
7. Formatter, standard library, and LSP after their contracts exist.

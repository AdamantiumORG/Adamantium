# Compiler architecture

Adamantium is organized as a Rust workspace with a CLI and focused compiler crates. The current compilation path is:

This is the starting document for contributors changing compiler behavior. The
[contributor guide](contributor-guide.md) maps common tasks to crates and test
commands, while [phase contracts](phases.md) define the invariants exchanged by
each stage.

```text
CLI and project loading
        |
lexer -> tokens
        |
parser -> AST
        |
name resolution -> SymbolId / DefId / LocalId
        |
type checking -> canonical TypeId
        |
HIR -> MIR -> target-independent IR
        |
x86-64 NASM generation
        |
NASM and platform linker
        |
native executable
```

The pipeline has three ownership zones:

| Zone | Responsibility | Must not do |
| --- | --- | --- |
| Frontend | Turn source into spanned, resolved, typed meaning | Select registers or invoke native tools |
| Middle end | Lower HIR through MIR into target-independent IR and optimize it | Parse source text or depend on an OS ABI |
| Backend and runtime | Select instructions, adapt platform ABIs, assemble, link, and execute runtime services | Reinterpret language syntax or type rules |

The current production implementation is transitional. Focused crates define
and test the intended boundaries, while mature language parsing, checking, and
x86-64 generation still live partly inside `adamantium-cli`. New work should
move data through crate APIs rather than adding another cross-layer dependency.

Parsing uses recursive descent for declarations, functions, classes, traits,
enums, and statements. Every expression is delegated to one Pratt parser with
central prefix, infix, and postfix parselet definitions. Calls, indexing, and
member access are postfix parselets; arithmetic, comparisons, logic, and
assignment are infix parselets. This keeps precedence and associativity in one
place as the operator set grows.

`adamantium-cli` currently owns the mature parser, semantic checker, and NASM generator while their public crate APIs are stabilized. The checker lowers parsed syntax into `adamantium_ir::typed::Program<Type, Value>`. `adamantium-ir` owns the target-independent control-flow, expression, class, package-call, and operator representation. It has no dependency on the parser or a native backend.

The existing NASM generator consumes this shared typed IR. A future LLVM ARM64 generator can consume the same IR without translating parser syntax or depending on x86-64 register conventions.

NASM was chosen for the first backend because its textual x86-64 output is easy
to inspect, snapshot, and diagnose while language semantics are changing. It
also makes stack layout, register selection, and Windows/System V calling
conventions explicit. The linker remains a separate phase so code generation
can be tested without producing an executable. LLVM support is being added for
architectures where maintaining another complete instruction selector would
slow development; both paths consume the same typed IR.

## Crate contracts

| Crate | Accepts | Produces | May depend on |
| --- | --- | --- | --- |
| `adamantium-lexer` | UTF-8 source | tokens or lexical errors | no compiler layer |
| `adamantium-parser` | tokens and source map | AST or parse errors | lexer, AST |
| `adamantium-semantics` | AST | resolved HIR and diagnostics | parser, HIR, canonical types |
| `adamantium-hir` | resolved language concepts | definitions using stable IDs | spans, canonical types |
| `adamantium-mir` | typed HIR | explicit control-flow operations | HIR, canonical types |
| `adamantium-lowering` | HIR or MIR | the next representation | HIR, MIR, IR |
| `adamantium-ir` | lowered operations | target-independent backend IR | spans, canonical types |
| `adamantium-codegen` | IR | target assembly | IR only |
| `adamantium-fmt` | source text | deterministic formatted source | no semantic layer |
| `adamantium-lsp` | JSON-RPC/LSP values | protocol capabilities and wire types | shared compiler APIs only |
| `adamantium-stdlib` | typed host-service requests | values or structured errors | no compiler layer |

Backends must not depend on the parser, AST, name resolution, or source text.
Every representation that survives a lowering boundary carries its original
`Span`. Human-readable names are interned once and referenced by `SymbolId`;
definitions and locals use `DefId` and `LocalId`. Types are interned once and
referenced by `TypeId`.

`Span` uses byte offsets into the original UTF-8 source. Tokens, identifiers,
AST expressions, resolved definitions, HIR definitions, MIR instructions and
IR instructions retain that span. Program nodes retain the complete source
span. Synthetic instructions use an empty span at the source location where
they are inserted, rather than the unrelated `0..0` default. Code generation
keeps an instruction-to-span map next to the emitted assembly so backend and
runtime diagnostics can still identify the originating source construct.

The exact input, output, invariants, and responsibilities of every stage are defined in [Compiler phase contracts](phases.md).

The generated program boundary, value ABI, allocation policy, error state, and
WASI isolation model are defined in [Runtime architecture](runtime.md).

## Workspace direction

The workspace contains focused crates including `adamantium-cli`, `adamantium-lexer`, `adamantium-parser`, `adamantium-ast`, `adamantium-semantics`, `adamantium-types`, `adamantium-diagnostics`, `adamantium-ir`, `adamantium-codegen`, `adamantium-nasm`, `adamantium-linker`, `adamantium-runtime`, `adamantium-project`, `adamantium-packages`, `adamantium-wasm`, `adamantium-testing`, `adamantium-stdlib`, `adamantium-fmt`, and `adamantium-lsp`.

Crates should be extracted only after the corresponding in-crate API has a clear input, output, ownership model, and diagnostic boundary. This avoids cyclic dependencies and prevents internal data structures from becoming public APIs accidentally.

Recommended extraction order:

1. AST and shared source locations.
2. Lexer and parser.
3. Diagnostics and type representation.
4. Semantic analysis.
5. Code generation, NASM invocation, and linking.
6. Project, package, WASM, and testing services.
7. Formatter, standard library, and LSP protocol and transport.

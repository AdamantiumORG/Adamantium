# Compiler phase contracts

This document defines the data passed between Adamantium compiler phases and the guarantees each phase may rely on. Some phases currently share an implementation module, but their responsibilities remain separate contracts.

## Pipeline

```text
project sources
    -> lexer
positioned tokens
    -> parser and generic expansion
syntax AST
    -> name resolution and syntax validation
resolved syntax Program
    -> type checking and semantic validation
typed IR Program<Type, Value>
    -> target backend
assembly or future LLVM IR
    -> assembler and linker
native executable
```

Diagnostics may stop the pipeline after any validating phase. A later phase must not repair invalid output from an earlier phase.

## Project loading

Input: a project directory containing `project.toml`, `requirement.toml`, `code/main.ad`, reachable packed modules, the lockfile, and installed package metadata.

Output: the canonical project root, validated project name, ordered `(module_name, source_text)` pairs, and package function bindings.

Responsibilities:

- Validate project and requirement manifests.
- Resolve module files and reject missing or circular `pack` dependencies.
- Load package declarations and lockfile state.
- Preserve source text unchanged for the frontend.

It does not tokenize, infer types, or generate native files.

## Lexer

Input: one UTF-8 Adamantium source string.

Output: `Vec<(Token, Position)>`, where every token carries its one-based starting line and column and the final item is exactly one `Token::End` sentinel.

Token guarantees:

- Words, numeric spelling, decoded string contents, punctuation, and end-of-input are distinct token variants.
- Whitespace and comments do not produce tokens.
- String escape sequences are decoded before parsing.
- Raw newlines, unknown escapes, unterminated strings, unterminated block comments, and unknown characters are lexer errors.
- Positions count source characters rather than UTF-8 bytes.

The lexer does not classify names, apply operator precedence, or inspect types.

## Parser and generic expansion

Input: positioned tokens from one or more modules.

Output: syntax declarations and expressions represented by the types in `syntax/ast.rs`. The project entry point returns `syntax::Program`.

Responsibilities:

- Recognize declarations, statements, expressions, types, attributes, visibility, modules, and imports.
- Apply expression precedence and associativity.
- Expand validated generic declarations into concrete specializations.
- Combine modules while retaining canonical symbol names.
- Attach source positions required by later diagnostics.

The parser does not choose machine registers, allocate runtime objects, or emit assembly.

## Syntax AST invariants

A successful `syntax::Program` guarantees:

- Function, class, enum, trait, import, and local declarations passed syntax validation.
- Every local variable expression contains a slot index valid for its function.
- A function's `types`, `positions`, and binding metadata use the same slot numbering.
- Class and enum numeric identifiers index the corresponding program metadata.
- Calls contain canonical function names and their parsed arguments.
- `break` and `continue` occur only inside loops, and `return` occurs inside a function.
- Assignments target changeable, live bindings.
- Module imports refer to declared public symbols of the expected symbol kind.
- Source positions identify the originating token for diagnostics.

The syntax AST may still contain unknown local types and type-invalid expressions. Those are type-checker responsibilities.

## Name resolution

Input: parsed module ASTs and their import relationships.

Output: the resolved `syntax::Program`. Name resolution is currently performed during parsing and module combination, so it does not yet have a separate Rust output type.

The resolved output guarantees:

- Local names are replaced by function-local slot indexes.
- Function and method calls use canonical names.
- Class and enum references use stable program-local numeric identifiers.
- `pack`, `use`, visibility, shadowing, removed names, and symbol aliases have been checked.
- Unknown, private, ambiguous, out-of-scope, and removed symbols have produced diagnostics instead of entering later phases.

Name resolution does not infer expression types or insert numeric conversions.

## Type checking

Input: a resolved `syntax::Program` and package signatures.

Output: `adamantium_ir::typed::Program<Type, Value>` through the concrete alias `typed::Program`.

The typed output guarantees:

- Every expression has one concrete `Type`.
- Every function slot has a concrete type.
- Calls have valid argument counts and compatible argument types.
- Explicit and implicit conversions are represented by typed IR nodes.
- Operators are valid for their operand types.
- Class fields, methods, lists, optionals, aliases, offsets, matches, and control-flow conditions passed their semantic type rules.
- Literal values are range checked and stored in the runtime `Value` representation where possible.
- Backend-independent operators use `adamantium-ir` enums rather than parser enums.

The type checker does not read files, select a target ABI, or emit native code.

## Semantic analysis

Semantic analysis is the collection of rules that require resolved names, types, or control-flow context. Today these checks are implemented across parser validation, `typed.rs`, and `diagnostics.rs`.

Input: syntax AST during early checks and typed IR during type-dependent checks.

Output: either validated data for the next phase or diagnostics. Advisory analysis additionally produces warnings without changing program meaning.

Responsibilities include:

- Visibility and import validation.
- Invalid access, assignment, conversion, alias, offset, class, and enum checks.
- Match exhaustiveness and unreachable-code checks.
- Unused function and variable warnings.
- Lifecycle, trait, generic constraint, and memory-safety rules.

Errors prevent code generation. Warnings do not mutate the AST or typed IR.

## Code generation

Input: only `adamantium_ir::typed::Program<Type, Value>` plus the selected entry function and target configuration.

The backend may rely on all type-checker guarantees. It must not parse source text, resolve names, infer types, or import parser operator types.

The current NASM backend produces x86-64 assembly. It lowers typed expressions and statements, calculates stack slots, applies the platform calling convention, and emits runtime calls. Windows and Linux ABI adaptation stays in the backend and runtime boundary. A future LLVM ARM64 backend must consume the same typed IR contract.

## Assembler and linker

Input: backend output, the Adamantium runtime library, target metadata, and required native system libraries.

Output: a native executable in the project's `target` directory.

This phase invokes NASM and the platform linker, reports tool failures as native-toolchain diagnostics, and does not change language semantics.

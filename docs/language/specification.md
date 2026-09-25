# Adamantium 0.1 language specification

This document freezes the core Adamantium language contract for the `0.1`
language series. The linked language chapters are normative. Compiler behavior
that contradicts these documents is a bug. Features explicitly described as
future work are outside this contract.

The freeze covers source programs, compile-time behavior, runtime-visible
behavior, project modules, packages, and tests. It does not freeze compiler
internals, diagnostics wording, generated assembly, optimizations, the package
download transport, or the future async language extension.

## Compatibility

Within the `0.1` series, a valid program must keep the same observable behavior
unless a change fixes undefined behavior or a documented compiler bug. New
syntax may be added only when it does not change the meaning of an existing
valid program. Removing syntax, changing a default type, changing value-copy or
alias behavior, or reserving an identifier requires a new language version.

The frozen source extension is `.ad`. Source is UTF-8. Statements end with `;`
unless they end in a block. Blocks use `{` and `}`. Line comments use `//`, and
block comments use `/*` and `*/`. Identifiers are case-sensitive.

## Reserved words

The frozen reserved-word set is:

```text
and assert break ch changeable class continue define else enum exit false for fun if
implements in List loop match not offset oofset or pack panic print priv pub
return static stc then trait true until use var variable warn while
```

`None`, built-in type names, and their sized forms are also unavailable as
identifiers. `var` and `variable`, `ch` and `changeable`, and `static` and `stc`
are intentional spelling aliases. `oofset` is retained for source
compatibility. Adding another reserved word requires a new language version.

## Types, inference, and conversions

The frozen type families are signed integers, unsigned integers, floating-point
values, booleans, strings, `None`, lists, enums, classes, optional values,
function values, aliases, and typed offsets. The supported scalar spellings and
runtime string and list rules are defined in [Types](types.md).

An unsuffixed integer literal is `i32`, and an unsuffixed decimal literal is
`f64`. `int`, `float`, and `u` select their documented default widths. A
non-empty list literal infers one common element type. Empty lists and ambiguous
expressions require an explicit type. Function parameters and named results do
not infer their declared types.

Implicit conversions do not silently change numeric families or widths.
Explicit `value.as(Type)` conversions are checked by the compiler and checked
again at runtime when the value can be outside the target range. `None` is
accepted only where an optional value is allowed. Enum, class, list, function,
alias, and offset values retain their declared identity and cannot be converted
to unrelated types.

## Functions and control flow

[Functions](functions.md) defines declarations, typed parameters, optional `$`
parameters, named results, calls, function values, visibility, and returns.
[Control flow](control-flow.md) defines `if`, loops, `match`, `break`,
`continue`, `return`, and process exit. Arguments are evaluated from left to
right. Ordinary values use value semantics. A function reaches its implicit
return by returning its named result.

## Classes, enums, generics, and traits

- [Classes](classes.md) defines construction, field and method visibility,
  `self`, value copying, and lifecycle hooks.
- [Enums](enums.md) defines variant identity and exhaustive matching.
- [Generics](generics.md) defines type parameters, specialization, and trait
  constraints.
- [Traits](traits.md) defines method requirements and `implements` validation.

These declarations are nominal. Separate classes, enums, and traits remain
different types even when their members have the same shape. Generic
specialization must preserve the same type and runtime checks as equivalent
non-generic code.

## Variables, memory, aliases, and offsets

[Variables](variables.md) defines mutability, lexical scope, shadowing, removal,
and normal value-copy assignment. [Memory and aliases](memory.md) and the
[memory-safety specification](../MEMORY_SAFETY.md) define ownership, object and
list copying, alias identity, synchronization, disconnection, detachment,
reattachment, removal, and offset lifetime rules.

Safe programs cannot observe dangling aliases or offsets, double-free storage,
or access removed values. An implementation may change its allocation strategy
only when these guarantees and observable lifecycle ordering remain unchanged.

## Modules and packages

[Modules](modules.md) freezes `pack`, `use`, qualified access, `mod`, and
visibility rules. Project functions, classes, and enums are private unless
declared `pub`; type aliases and traits are exported declarations. Imports from
another module or package may resolve only public symbols.

Package declarations, versions, locking, caching, checksums, and publishing are
defined by [Creating packages](../CREATING_PACKAGES.md). Executable package
behavior uses the frozen [Adamantium WASM package ABI](../packages/WASM_ABI.md).
Transport and cache layout may evolve without changing source or ABI behavior.

## Errors and tests

[Errors and diagnostics](errors.md) freezes the distinction between compile-time
errors, recoverable runtime errors, warnings, and panics. Stable error codes and
runtime error kinds are part of the contract. Diagnostic prose and highlighting
may improve without a language-version change.

[Testing Adamantium](../TESTING.md) freezes `tests.ad`, `#[test]`, test discovery,
filtering, result status, and the `adamantium test` command family. Tests execute
with the same language and runtime semantics as ordinary project functions.

[Decorators](decorators.md) define function entry hooks, their top-to-bottom
execution order, callback arguments, class inheritance, and method exclusions.
Decorator calls use the same name, arity, and type checks as ordinary calls.

## Change process

Every proposed language change must include parser, semantic, code-generation,
runtime, invalid-program, and documentation coverage where applicable. A change
that modifies this contract must update the language version and include a
migration note. Compiler refactoring alone does not change the language version.

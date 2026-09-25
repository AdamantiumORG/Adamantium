# Parser

The Adamantium frontend is organized under `src/syntax`:

```text
syntax/
|-- mod.rs          stable frontend exports and tests
|-- tokens.rs       token definitions and positioned token aliases
|-- diagnostics.rs  source positions and parser error formatting
|-- lexer.rs        source text to positioned tokens
|-- ast.rs          syntax-level expressions, statements and declarations
`-- parser.rs       parsing, generic expansion, modules and validation
```

The public compiler entry points remain `parse_modules`, `module_dependencies`, and `package_dependencies`. Tests also use `parse` for single-source programs. Callers continue to receive the same syntax-level `Program` and `line:column` diagnostics as before the split.

Statement and declaration parsing uses recursive descent. Expression parsing
uses a Pratt parser and a single operator contract:

```text
prefix:  -  !  not
postfix: call()  index[]  .member
infix:   assignment, logic, equality, comparison, addition, multiplication
```

Postfix operators bind most tightly. Multiplication binds more tightly than
addition, followed by comparisons, equality, logical AND, logical OR, and
right-associative assignment. New expression operators must be added to the
central parselet definitions rather than through another expression parser.

Nested blocks have lexical scopes. Names declared in a child block expire at its closing brace, while parent bindings remain accessible. Local variables may shadow parent variables. Classes, enums, traits, type aliases, and imported symbols have protected namespace rules.

Module combination and generic expansion remain in `parser.rs` because they currently share parser state and token rewriting. They can be extracted after those stages receive independent input and output contracts.

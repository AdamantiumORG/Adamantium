# Core stability and compiler hardening

Adamantium 0.1 freezes its core syntax, primitive type semantics and memory
semantics. The normative contracts are the language specification, the type
reference and the memory-safety specification. Changes that alter existing
program behavior require a language version change and migration notes.

Compiler diagnostics may improve without changing the language. Stable error
codes identify diagnostic families. Invalid source must return a diagnostic and
must never expose a Rust panic to the user. The CLI has a final panic boundary
for unexpected internal failures, while regression tests exercise malformed and
generated input directly against the frontend.

The checked-in language suite covers valid and invalid programs. A deterministic
10,000-input mutation corpus runs with `cargo test`. The `fuzz/frontend` target
provides continuous coverage-guided fuzzing with `cargo-fuzz` for normal and
Professional Mode parsing.

Parser recovery that continues after multiple grammar errors in one module is
still under development. Until it is complete, the compiler guarantees a clear,
located diagnostic for the first grammar error and aggregates errors in phases
that already support independent validation.

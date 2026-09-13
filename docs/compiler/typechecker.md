# Type checker

`src/typed.rs` validates the syntax program and produces typed expressions and instructions for code generation. It owns type inference, conversions, call signatures, class fields, list element checks, optional values, offsets, operators, and typed control flow.

The type checker receives resolved slot and symbol identifiers. It should not read project files or emit assembly. Errors retain source positions supplied by the syntax layer and are rendered by the diagnostics module.

The future semantics and types crates should separate name resolution from type rules before `typed.rs` is split.

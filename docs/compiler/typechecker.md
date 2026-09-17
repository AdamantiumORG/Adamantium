# Type checker

`src/typed.rs` validates the resolved syntax program and produces `adamantium_ir::typed::Program<Type, Value>` for code generation. It owns type inference, conversions, call signatures, class fields, list element checks, optional values, offsets, operators, and typed control flow.

The type checker receives resolved slot and symbol identifiers. It should not read project files or emit assembly. Errors retain source positions supplied by the syntax layer and are rendered by the diagnostics module.

The complete input and output guarantees are defined in [Compiler phase contracts](phases.md). The future semantics and types crates should preserve those contracts while separating name resolution from type rules.

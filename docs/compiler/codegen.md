# Code generation

`src/codegen` converts the typed program into x86-64 NASM source. It does not parse source text or resolve names.

```text
codegen/
├── mod.rs          shared generator state and public entry export
├── assembly.rs     slots, calls, runtime calls, printing and evaluation
├── expressions.rs  typed expression lowering
├── statements.rs   statements and control flow
├── functions.rs    function frames and saved calls
├── lists.rs        list bounds checks
└── entry.rs        CLI entry point, data section and platform adaptation
```

Every Adamantium value occupies a 16-byte slot. The generator tracks temporary slots and calculates the largest stack frame used by each function. Typed expressions leave their value in `rax` and `rdx`. Runtime failures branch to the active error target.

Windows and Linux share generated logic. `entry.rs` maps runtime calls to Linux ABI wrappers when producing Linux assembly. Platform-specific assembly remains in `src/runtime.asm` and `src/runtime-linux.asm`.

Future splits should follow actual responsibilities. Class construction, operators, and runtime request encoding can move to separate files when they grow enough to justify another boundary.

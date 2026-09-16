# Code generation

`src/codegen` converts `adamantium_ir::typed::Program<Type, Value>` into x86-64 NASM source. It does not parse source text or resolve names. Typed IR operators and instructions are target-independent; register selection, stack layout, calling conventions, and assembly syntax belong to the backend.

```text
codegen/
├── mod.rs          shared generator state and public entry export
├── assembly.rs     slots, calls, runtime calls, printing and evaluation
├── expressions.rs  typed expression lowering
├── statements.rs   statements and control flow
├── functions.rs    function frames and saved calls
├── lists.rs        recursive List copies and bounds checks
└── entry.rs        CLI entry point, data section and platform adaptation
```

Every Adamantium value occupies a 16-byte slot. The generator tracks temporary slots and calculates the largest stack frame used by each function. Typed expressions leave their value in `rax` and `rdx`. Runtime failures branch to the active error target.

Windows and Linux share generated logic. `entry.rs` maps runtime calls to Linux ABI wrappers when producing Linux assembly. Platform-specific assembly remains in `src/runtime.asm` and `src/runtime-linux.asm`.

Future splits should follow actual responsibilities. Class construction, operators, and runtime request encoding can move to separate files when they grow enough to justify another boundary.

## LLVM ARM64 boundary

The LLVM backend should accept the same typed `Program` used by NASM and lower it to LLVM IR. It must not import parser AST types. Target triples, data layouts, ABI lowering, object emission, and linker selection stay outside `adamantium-ir`. This keeps semantic behavior shared while allowing Windows ARM64 and Linux ARM64 to use their platform ABIs.

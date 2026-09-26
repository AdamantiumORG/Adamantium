# Code generation

`src/codegen` converts `adamantium_ir::typed::Program<Type, Value>` into x86-64 NASM source. It does not parse source text or resolve names. Typed IR operators and instructions are target-independent; register selection, stack layout, calling conventions, and assembly syntax belong to the backend.

```text
codegen/
|-- mod.rs            shared generator state and public entry export
|-- assembly.rs       labels, slots and basic assembly emission
|-- expressions.rs    expression dispatch
|-- statements.rs     statement dispatch
|-- classes.rs        class construction, fields, methods and copying
|-- lists.rs          recursive List copies and bounds checks
|-- control_flow.rs   branches, loops and match lowering
|-- operators.rs      unary, binary, logical and comparison operators
|-- runtime_calls.rs  runtime requests, printing and function calls
|-- functions.rs      function frames and saved calls
`-- entry.rs          CLI entry point, data section and platform adaptation
```

Every Adamantium value occupies a 16-byte slot. The generator tracks temporary slots and calculates the largest stack frame used by each function. Typed expressions leave their value in `rax` and `rdx`. Runtime failures branch to the active error target.

Windows, Linux, and Intel macOS share generated logic. `entry.rs` maps runtime calls to System V ABI wrappers for Linux and macOS. Platform-specific assembly remains in `src/runtime.asm`, `src/runtime-linux.asm`, and `src/runtime-macos.asm`.

Expression and statement modules dispatch typed IR nodes to focused lowering modules. Runtime request layout remains in `runtime_calls.rs`, while target-specific call-name and ABI adaptation remains in `entry.rs` and the platform runtime assembly files.

## LLVM ARM64 boundary

The LLVM backend should accept the same typed `Program` used by NASM and lower it to LLVM IR. It must not import parser AST types. Target triples, data layouts, ABI lowering, object emission, and linker selection stay outside `adamantium-ir`. This keeps semantic behavior shared while allowing Windows ARM64 and Linux ARM64 to use their platform ABIs.

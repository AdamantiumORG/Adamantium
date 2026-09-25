# Adamantium compilation targets

## Supported targets

| Target | Object format | Calling convention | Linker | Status |
| --- | --- | --- | --- | --- |
| `x86_64-pc-windows-msvc` | COFF (`win64`) | Windows x64 for runtime calls | MSVC `link.exe` | Supported |
| `x86_64-unknown-linux-gnu` | ELF64 | System V AMD64 for runtime calls | `cc` or bundled Zig | Supported |

Adamantium function calls use the compiler's internal stack-based value ABI on
both targets. Platform entry points and calls into the Rust runtime use adapters
for the operating system ABI.

## ARM64 backend

The LLVM backend recognizes these target names:

* `aarch64-pc-windows-msvc`
* `aarch64-unknown-linux-gnu`
* `aarch64-apple-darwin`

The ARM64 backend emits LLVM IR directly from target-independent IR. It does
not translate generated x86-64 assembly text. Windows and Linux use separate
target triples and data layouts. Object generation invokes Clang with an
explicit target, and linker argument construction selects ARM64 explicitly.
Every runtime `Value` remains 16 bytes, with two 64-bit words, and keeps the
same runtime type identifiers.

The core LLVM emitter is implemented. Portable ARM64 distributions remain
blocked until the mature CLI lowering path feeds all language operations into
the shared IR and native ARM64 language tests pass. An ARM64 CLI archive that
cannot compile Adamantium programs is not considered a supported portable
distribution.

The `ARM64 backend` workflow runs on native Windows ARM64 and Linux ARM64
runners. It emits LLVM IR through `adamantium-codegen`, lets Clang create the
platform executable, and runs that executable on the matching architecture.

Before an ARM64 target can be marked supported, it must provide:

* A platform entry point and command-line argument adapter.
* AAPCS64-compatible runtime calls and 16-byte stack alignment.
* COFF, ELF or Mach-O object generation for the selected operating system.
* A linker implementation and runtime static library for that target.
* Native tests for arithmetic, functions, classes, lists, errors and exit codes.
* CI that builds and runs generated programs on real ARM64 hardware or an
  explicitly documented emulator.

Target selection will eventually be exposed through a `--target <triple>` CLI
option. Until that option exists, the compiler builds programs for its host
operating system and x86-64 architecture.

## Portable Linux distribution

`linux_portable_x86_64.zip` contains the Adamantium CLI, NASM, and Zig's linker
toolchain. The CLI discovers both tools relative to its own executable. The
packaging smoke test removes system compiler tools from `PATH`, creates a new
project, compiles it, and runs the resulting ELF executable before publication.

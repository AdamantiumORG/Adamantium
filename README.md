[![CI](https://github.com/AdamantiumORG/Adamantium/actions/workflows/ci.yml/badge.svg)](https://github.com/AdamantiumORG/Adamantium/actions/workflows/ci.yml)
[![Linux CLI](https://github.com/AdamantiumORG/Adamantium/actions/workflows/cli-linux.yml/badge.svg)](https://github.com/AdamantiumORG/Adamantium/actions/workflows/cli-linux.yml)
[![Windows CLI](https://github.com/AdamantiumORG/Adamantium/actions/workflows/cli-windows.yml/badge.svg)](https://github.com/AdamantiumORG/Adamantium/actions/workflows/cli-windows.yml)
[![License: GPL](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://opensource.org/licenses/gpl-3-0)

# Adamantium

## What is Adamantium?

Adamantium is a programming language for beginners and experienced developers. Its compiler is written in Rust and translates Adamantium source code into NASM assembly, then builds native Windows or Linux x86-64 executables.

```adamantium
fun main() {
    print.newline("Hello from Adamantium!");
}
```

## Why Adamantium?

Adamantium aims to provide readable syntax while still exposing functions, classes, enums, traits, generics, modules, explicit types, memory operations, and native compilation. Projects use a predictable directory structure and can be checked, built, run, and tested through one CLI.

## Hello World

Create `code/main.ad`:

```adamantium
fun main() {
    print.newline("Hello, world!");
}
```

Run it from the project directory:

```text
adamantium run
```

## Features

- Mutable and static variables with type inference
- Integers, floating-point values, strings, booleans, lists, and optional values
- Functions with typed parameters and named return values
- Classes, visibility rules, methods, and lifecycle hooks
- Enums, traits, generics, and operator overloading
- `if`, `match`, `for`, `while`, `until`, and `loop`
- Nested lexical scopes
- Multiple source files with `pack` and `use`
- WASI packages loaded with `mod`
- Value aliases, offsets, and memory-safety diagnostics
- Project tests and file-based language conformance tests
- Native Windows and Linux x86-64 output

## Installation

Build and install the CLI from source:

```text
cargo install --path crates/adamantium-cli
```

For Windows x86-64, download `windows_portable_x86_64.zip` from the [Nightly release](https://github.com/AdamantiumORG/Adamantium/releases/tag/nightly), extract the complete directory, and add that directory to `PATH`. The portable package includes the CLI, NASM, the LLVM linker, and the required Windows import libraries. It does not require Rust, Cargo, Visual Studio, the Windows SDK, or a separate NASM installation. Nightly is replaced automatically after all Windows tests pass on the default branch.

For Linux x86-64, download `linux_portable_x86_64.zip` from the same Nightly release. It includes the CLI, NASM, and the Zig linker toolchain, so Rust, Cargo, NASM, GCC, Clang, and system development packages are not required. The Linux archive is rebuilt and its Nightly assets are replaced after all Linux CLI tests pass on the default branch.

When building the CLI from source, building Adamantium programs requires NASM and a platform linker:

- Windows: Visual Studio C++ Build Tools and the Windows SDK
- Linux: a C linker available as `cc`

Each portable archive contains `INSTALL.txt`, and a SHA-256 checksum is published beside it.

## Quick Start

Create a project:

```text
adamantium new FirstProject
cd FirstProject
adamantium check
adamantium run
```

The generated project contains:

```text
FirstProject/
├── code/
│   └── main.ad
├── target/
├── project.toml
└── requirement.toml
```

Useful commands:

```text
adamantium check
adamantium build
adamantium run
adamantium test list
adamantium test run
adamantium install
adamantium clean
```

## Example

```adamantium
enum Operation { add, multiply }

fun calculate(a:int,b:int,operation:Operation) result:int {
    match operation {
        Operation.add => { result=a+b; },
        Operation.multiply => { result=a*b; }
    }
}

fun main() {
    var values=List[2,3,4];
    var result=calculate(values[0],values[1],Operation.multiply);

    if result > 5 then {
        print.newline(result);
    }
}
```

## Project Status

Adamantium is under active development. The compiler currently supports Windows and Linux on x86-64. Language behavior, package ABI, memory-safety rules, diagnostics, and compiler architecture are still being expanded and may change before the first stable release.

Detailed language, compiler, package, testing, memory-safety, and target documentation is available in [`docs`](docs/README.md). Current and planned work is tracked in [`TODO.md`](TODO.md).

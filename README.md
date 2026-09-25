[![CI](https://github.com/AdamantiumORG/Adamantium/actions/workflows/ci.yml/badge.svg)](https://github.com/AdamantiumORG/Adamantium/actions/workflows/ci.yml)
[![License: GPL](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://opensource.org/licenses/gpl-3-0)

# Adamantium

## What is Adamantium?

Adamantium is a programming language for beginners and experienced developers. Its compiler is written in Rust and translates Adamantium source code into NASM assembly, then builds native Windows or Linux x86-64 executables.

```adamantium
fun fibonacci(n: i32) r: i32 {
    var a = 0;
    var b = 1;
    var i = 0;

    while i < n {
        print(a);

        var next = a + b;
        a = b;
        b = next;
        i = i + 1;
    }

    r = a;
}

fun main() {
    fibonacci(10);
}
```

## Why Adamantium?

Adamantium aims to provide readable syntax while exposing functions, classes, enums, traits, generics, modules, explicit types, memory operations, and native compilation. Projects use a predictable directory structure and can be checked, built, run, and tested through one CLI. The project is pre-1.0, so advanced features remain experimental even when they are implemented and tested.

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
- Configurable `-O0`, `-O1`, and `-O2` optimization levels

## Installation

Build and install the CLI from source:

```text
cargo install --path crates/adamantium-cli
```

For Windows x86-64, download `windows_portable_x86_64.zip` from the [Nightly release](https://github.com/AdamantiumORG/Adamantium/releases/tag/nightly), extract the complete directory, and add that directory to `PATH`. The portable package includes the CLI, NASM, the LLVM linker, and the required Windows import libraries. It does not require Rust, Cargo, Visual Studio, the Windows SDK, or a separate NASM installation. Nightly is replaced automatically after all Windows tests pass on the default branch.

For Linux x86-64, download `linux_portable_x86_64.zip` from the same Nightly release. It includes the CLI, NASM, and the Zig linker toolchain, so Rust, Cargo, NASM, GCC, Clang, and system development packages are not required. The Linux archive is rebuilt and its Nightly assets are replaced after all Linux CLI tests pass on the default branch.

Building the Rust CLI from source requires the normal Rust host toolchain. Building Adamantium programs with that CLI requires NASM and a supported linker configuration:

- Windows: bundled or configured `rust-lld`, or another linker selected by the CLI
- Linux: a C linker available as `cc`, or the portable Zig linker toolchain

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
adamantium build -O2
adamantium run
adamantium test list
adamantium test run
adamantium install
adamantium clean
adamantium doctor
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

Adamantium is under active development. Native and portable builds target Windows and Linux on x86-64, with ARM64 work tracked separately. The 0.1 core syntax, primitive types, and memory rules are documented as frozen contracts, while advanced language features, the package ABI, diagnostics, and implementation architecture remain experimental before the first stable release. Adamantium has not completed an independent security audit.

Detailed language, compiler, package, testing, memory-safety, and target documentation is available in [`docs`](docs/README.md). Current and planned compiler work is tracked in [`TODO.md`](TODO.md). Security reports follow [`SECURITY.md`](SECURITY.md).

[![CI](https://github.com/AdamantiumORG/Adamantium/actions/workflows/ci.yml/badge.svg)](https://github.com/AdamantiumORG/Adamantium/actions/workflows/ci.yml)
[![License: GPLv3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE.md)
[![Status: Early development](https://img.shields.io/badge/status-early_development-orange.svg)](#project-status)

# Adamantium

**A simple native programming language for people who want readable syntax
without giving up explicit control.**

[Install](#installation) | [Documentation](docs/README.md) | [Try online](docs/PLAYGROUND.md) | [Versioning](docs/VERSIONING.md) | [Examples](examples/README.md) | [Contributing](CONTRIBUTING.md)

Adamantium is an experiment in building an approachable, statically typed
language from scratch. Its Rust compiler turns `.ad` source into NASM assembly
and then into native Windows or Linux x86-64 executables.

```text
$ adamantium new HelloWorld
$ cd HelloWorld
$ adamantium run
Hello, world!
```

```adamantium
fun main() {
    var numbers = List[1, 2, 3, 4, 5];

    for number in numbers {
        print.newline(number);
    }
}
```

```text
Adamantium source -> compiler written in Rust -> NASM -> native x86-64 executable
```

Adamantium started with NASM because textual x86-64 output is easy to inspect,
compare in tests, and debug while the language semantics are still evolving.
NASM also keeps instruction selection and calling conventions visible to people
learning how the compiler works. The platform linker combines that object code
with the Rust runtime. Target-independent typed IR now separates language
semantics from this backend, so LLVM can serve ARM64 and future targets without
forcing the original x86-64 path to be rewritten first. See the
[compiler architecture](docs/compiler/architecture.md) for the phase boundaries.

## Why Adamantium?

Why build another programming language? Adamantium explores a space between
high-level readability and native compilation. It does not aim to replace an
established language. It is a place to test whether explicit types, native code,
memory operations, and modern language features can remain approachable.

| Language | Strengths | Typical tradeoff |
| --- | --- | --- |
| C and C++ | Mature ecosystems and direct machine control | Large, complex languages with many unsafe edges |
| Python | Easy to learn and productive | Higher runtime abstraction and dynamic typing |
| Rust | Strong safety model and powerful type system | A steeper learning curve |
| Adamantium | Readable syntax, static typing, native compilation, and explicit control | Young ecosystem and experimental pre-1.0 design |

Adamantium is for people interested in language design, compiler construction,
and native programs who want to learn by reading, running, and changing real
code.

## Language tour

Functions use explicit parameters and a named result:

```adamantium
fun fibonacci(n:int) result:int {
    if n <= 1 {
        result = n;
    } else {
        result = fibonacci(n-1) + fibonacci(n-2);
    }
}
```

Enums work with exhaustive `match` expressions:

```adamantium
enum Status { ready, running, finished }

fun show(status:Status) result:None {
    match status {
        Status.ready => { print.newline("Ready"); }
        Status.running => { print.newline("Running"); }
        Status.finished => { print.newline("Finished"); }
    }
}
```

Generic functions and trait-constrained classes are part of the current
experimental language:

```adamantium
fun identity<T>(value:T) result:T {
    result = value;
}

trait Named {
    fun name() result:string;
}

class Item(pub text:string) implements Named {
    fun __new__() {}
    pub fun name() result:string { result=self.text; }
}
```

Explore complete projects in [`examples`](examples/README.md).

## Features

- A written [language specification](docs/language/specification.md) covering the core syntax and semantics
- Native Windows, Linux, and macOS x86-64 executables
- Static types with inference and Professional Mode with explicit declarations
- Functions, classes, enums, traits, generics, lists, and optional values
- `if`, exhaustive `match`, `for`, `while`, `until`, and `loop`
- Modules through `pack` and `use`
- Checked aliases, offsets, lifecycle hooks, and memory diagnostics
- WASI packages with explicit imports, integrity checks, and runtime limits
- Built-in project and language test runners
- `-O0`, `-O1`, and `-O2` optimization profiles
- Structured diagnostics with source spans and stable error families

## Installation

The easiest installation is a tested archive from one of the release channels:

- Stable `vMAJOR.MINOR.PATCH` releases are immutable and intended for normal use.
- Development `dev` is updated after the complete CI and CLI suite passes.
- Nightly `nightly` follows the newest tested commit.

See the [versioning and release policy](docs/VERSIONING.md) and
[changelog](CHANGELOG.md).

- `windows_portable_x86_64.zip` includes the CLI, NASM, LLVM linker, and import libraries.
- `linux_portable_x86_64.zip` includes the CLI, NASM, and Zig linker toolchain.
- Stable Windows releases also provide `adamantium-windows-x86_64-installer.exe`, which installs the same self-contained toolchain and updates the user `PATH`.

Extract the complete archive and add its directory to `PATH`. Verify the
published SHA-256 file before use. Rust and separate native build tools are not
required by the portable archives.

To build the CLI from source:

```text
cargo install --path crates/adamantium-cli --locked
adamantium doctor
```

Source builds use the host Rust toolchain. Compiling Adamantium programs also
needs NASM and a supported linker configuration. Rust 1.85 or newer is required
because the workspace uses Rust 2024 edition. CI tests the current stable Rust
release.

If `cargo install` succeeds but Adamantium cannot build a program, run
`adamantium doctor`. Install NASM and the platform linker reported as missing,
then make sure both are available through `PATH`. On Windows, use the Visual
Studio x64 native tools environment or the portable archive, which already
contains NASM, LLVM's linker, and the required import libraries. On Linux,
install NASM and a C linker or use the portable archive with its bundled Zig
toolchain. On macOS, install NASM and the Xcode Command Line Tools.

### Supported platforms

| Architecture | Windows | Linux | macOS |
| --- | --- | --- | --- |
| x86-64 | Supported | Supported | Supported |
| ARM64 | Experimental backend and CI smoke tests | Experimental backend and CI smoke tests | Not implemented |

Portable release archives currently target x86-64. The ARM64 LLVM backend is
under development and is not yet a complete replacement for the x86-64 CLI
pipeline. See [compilation targets](docs/TARGETS.md) for exact status.

## Quick start

[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://codespaces.new/AdamantiumORG/Adamantium?quickstart=1)

```text
adamantium new FirstProject
cd FirstProject
adamantium check
adamantium run
```

Generated project layout:

```text
FirstProject/
|-- code/
|   `-- main.ad
|-- target/
|-- project.toml
`-- requirement.toml
```

Common commands:

```text
adamantium check
adamantium build -O2
adamantium run
adamantium test list
adamantium test run
adamantium install
adamantium clean
adamantium doctor
```

Errors can be reported, handled, or emitted as non-fatal warnings:

```adamantium
fun main() {
    warn("Using the fallback configuration");

    var error = try {
        panic("The requested operation failed");
    };

    if error != None {
        print.newline(error);
    }
}
```

See [errors and diagnostics](docs/language/errors.md) for the runtime behavior
of `try`, `panic`, and `warn`.

## Known limitations

- Adamantium is pre-1.0 and its advanced language and package APIs may change.
- It has not completed an independent security audit and should not yet be used for security-critical production systems.
- Portable releases are currently available only for x86-64.
- ARM64 supports an experimental LLVM path and packaging smoke tests, but not the complete language and portable toolchain.
- Package checksums detect corrupted or mismatched assets but do not authenticate publishers; package signatures are still planned.
- The standard library, package ecosystem, editor integrations, and debugger support are still developing.

## Roadmap

- [x] Scanner lexer, parser, AST, name resolution, and type checking
- [x] NASM x86-64 backend and native Windows/Linux/macOS executables
- [x] Classes, enums, traits, generics, modules, tests, and optimization profiles
- [x] WASI package format, dependency locking, checksums, and execution limits
- [ ] Broader standard library and package ecosystem
- [ ] Language server and editor integration
- [ ] Debugger integration
- [ ] Stabilized compatibility policy and 1.0 release

The detailed engineering backlog is in [`TODO.md`](TODO.md).

## Project status

Adamantium is in early development. The 0.1 core syntax, primitive types, and
memory rules have written contracts, while advanced features and tooling remain
experimental. The project has not completed an independent security audit.

The best way to help is to run an example, report a focused issue, improve a
diagnostic, or discuss a concrete language-design tradeoff. See
[`CONTRIBUTING.md`](CONTRIBUTING.md) and [`SECURITY.md`](SECURITY.md).

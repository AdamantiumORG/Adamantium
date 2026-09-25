# Standard library

The official standard library is implemented by the `adamantium-stdlib` crate.
It keeps host operating-system services outside the lexer, parser, semantic
checker, and native backends. Language bindings and package adapters must call
this crate instead of duplicating platform-specific behavior.

## Modules

| Module | Contract |
| --- | --- |
| `collections` | Search, reverse, and sort collection values without changing the input |
| `time` | Unix time in milliseconds, monotonic elapsed time, and sleeping |
| `random` | Process-seeded random values, deterministic seeding, and half-open ranges |
| `environment` | Read environment variables and the current directory |
| `process` | Run a program directly, with explicit arguments, directory, and environment |
| `network` | Resolve an address and perform a bounded TCP exchange with timeouts |
| `filesystem` | Read, write, append, list, create, test, and remove paths |
| `json` | Parse, validate, compact, and pretty-print JSON |
| `asynchronous` | Spawn a task and join it while converting task panics into errors |
| `testing` | Equality and boolean assertions with readable failures |

String, math, and recoverable-error behavior continue to live in the core
runtime because generated programs already use those operations directly.

## Error contract

Fallible operations return `adamantium_stdlib::Result<T>`. Its error contains a
stable operation name and a platform error message. The stable name lets the
language runtime classify errors without parsing operating-system text.

The library never invokes a command shell. Process arguments are passed as
individual values. Network reads have a caller-provided byte limit and both
connect and I/O operations have a timeout. These limits are part of the public
contract.

## Cross-platform behavior

Paths use the host platform's native representation. Text is UTF-8. Directory
list results are sorted to give deterministic behavior. Environment values and
process output that are not valid UTF-8 use a lossy conversion rather than
causing a panic. Missing variables return `None`.

Random ranges use `start..end`, so `end` is excluded. An empty or reversed
range returns `None`. JSON follows the JSON data model provided by
`serde_json`, including objects, arrays, strings, numbers, booleans, and null.

## Architecture boundary

The crate exposes Rust contracts used by runtime and package adapters. It does
not parse Adamantium source and it does not generate assembly. This boundary
allows the x86-64 and future ARM64 backends to share the same standard-library
semantics.


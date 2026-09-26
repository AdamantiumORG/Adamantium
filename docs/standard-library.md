# Standard library specification

The official standard library is implemented by the `adamantium-stdlib` crate.
It keeps host operating-system services outside the lexer, parser, semantic
checker, and native backends. Language bindings and package adapters must call
this crate instead of duplicating platform-specific behavior.

This document is the normative pre-1.0 contract for contributors. Public
operations use UTF-8 text, never panic for expected host failures, and return
the shared `Result<T>` type unless their signature explicitly returns `Option`
or an infallible value. New modules must follow the same error and portability
rules and include tests on every supported host.

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

## Public API

The names below are the canonical host API. Adamantium source bindings may use
language-style names, but must preserve these arguments, results, and effects.

| Module | Operations |
| --- | --- |
| `collections` | `contains`, `index_of`, `reversed`, `sorted` |
| `time` | `unix_milliseconds`, `sleep_milliseconds`, `Stopwatch.start`, `Stopwatch.elapsed_milliseconds` |
| `random` | `seed`, `next_u64`, `range(start, end)` |
| `environment` | `get`, `exists`, `current_directory` |
| `process` | `Command.new`, `argument`, `directory`, `environment`, `run` |
| `network` | `tcp_exchange(address, payload, timeout, maximum_response_bytes)` |
| `filesystem` | `read_text`, `write_text`, `append_text`, `create_directory`, `remove`, `exists`, `list` |
| `json` | `parse`, `compact`, `pretty` |
| `asynchronous` | `Task.spawn`, `Task.join` |
| `testing` | `equal`, `truthy` |

Collection transformations return new values and do not modify their input.
`process.Command` executes the program directly without a shell. `Task.join`
can be called once and converts a worker panic into a standard-library error.

## Networking example

The current networking contract is exposed to runtime and package adapters by
the Rust standard-library crate. This example sends a complete HTTP request,
waits at most two seconds, and reads at most 64 KiB:

```rust
use adamantium_stdlib::network::tcp_exchange;
use std::time::Duration;

let response = tcp_exchange(
    "example.com:80",
    b"GET / HTTP/1.0\r\nHost: example.com\r\n\r\n",
    Duration::from_secs(2),
    64 * 1024,
)?;
```

The operation resolves one endpoint, establishes one TCP connection, writes
the full payload, then reads until EOF or the byte limit. DNS, connection,
timeout, write, and read failures retain distinct operation identifiers.
Direct Adamantium syntax for this module is not stable yet; documentation must
not present a proposed binding as an implemented language feature.

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

## Contribution rules

Every public operation requires deterministic semantics where the host allows
them, a stable error operation name, API documentation, unit tests, and a
cross-platform test when it touches the operating system. Platform-specific
extensions belong behind a portable contract or in an external package. A
breaking signature or semantic change follows the
[language specification](language/specification.md) and the
[project versioning policy](VERSIONING.md).

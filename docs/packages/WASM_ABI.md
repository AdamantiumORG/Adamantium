# Adamantium WASM package ABI

## ABI identifier

The supported ABI is `wasi-command-v1`. A package is a WebAssembly 1.0 core module using WASI Preview 1 through the `wasi_snapshot_preview1` import namespace. Components and imports from other host namespaces are rejected.

The module must export a function named `_start` with no parameters and no result. Adamantium validates the complete WebAssembly binary and required export before installation, project analysis, and execution.

## Initialization and function discovery

Adamantium creates a fresh WASI instance for every package call. Instantiation initializes the module's memory and globals. The runtime then calls `_start` once. Packages must not rely on state surviving between calls.

Public Adamantium functions are discovered from the manifest's `[functions]` table. Each entry maps an Adamantium function name to a command. The `command` field defaults to the function name. WebAssembly function exports are inspected for validation and tooling, while `_start` is the only function export invoked by this ABI.

## Call protocol

The runtime supplies arguments as UTF-8 command-line values:

```text
argv[0] = adamantium-packet
argv[1] = manifest command
argv[2..] = Adamantium arguments
```

The package writes its return value to stdout. A successful call exits with status zero. A result of `None` ignores stdout. String results preserve stdout exactly. Numeric and boolean results trim surrounding whitespace before conversion.

## Supported values

| Adamantium type | Argument encoding | Result encoding |
| --- | --- | --- |
| `i8`, `i16`, `i32`, `i64` | signed decimal UTF-8 | signed decimal UTF-8 |
| `u4`, `u8`, `u16`, `u32`, `u64` | unsigned decimal UTF-8 | unsigned decimal UTF-8 |
| `f32`, `f64` | decimal UTF-8 | decimal UTF-8 |
| `bool` | `true` or `false` | `true` or `false` |
| `string` | unchanged UTF-8 | unchanged stdout bytes interpreted as UTF-8 |
| `None` | not allowed | stdout ignored |

Each function accepts at most eight arguments. `f128`, lists, classes, enums, optional values, offsets, and generic runtime values are not supported in `wasi-command-v1`.

## Filesystem capability

`permissions.filesystem` can be `none`, `read`, or `read-write`. `none` exposes no project directory. The other modes preopen the current project directory as `.`. In this ABI, `read` is advisory because the WASI Preview 1 backend does not independently remove write rights.

## Errors

A package reports failure by writing a UTF-8 message to stderr and returning a nonzero WASI exit status or trapping. Adamantium reports the message as `Adamantium package error`. If stderr is empty, the runtime reports the WASM trap or loader error. Package errors participate in Adamantium `try` handling. Invalid modules, unsupported imports, invalid return values, invalid UTF-8, and permission errors fail the call instead of producing a value.

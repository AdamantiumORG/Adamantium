# Runtime architecture

This document defines the runtime used by generated Adamantium programs. It is
the contract between native code generation and `adamantium-runtime`.

## Responsibilities

The compiler owns parsing, type checking, object layout, stack-slot selection,
and control flow. The runtime provides operations that need host services or
would be unsafe to duplicate in generated assembly:

- process startup and command-line argument conversion;
- printing, warnings, panics, and recoverable errors;
- checked arithmetic and explicit conversions;
- object and List storage allocation and copying;
- UTF-8 string operations;
- isolated WASI package execution.

Runtime entry points use a stable C ABI. Platform assembly adapters translate
the compiler's internal x86-64 register convention to the Windows or System V
ABI. The Rust runtime does not parse Adamantium source or make language-level
type-inference decisions.

## Values and calls

Every runtime `Value` is 16 bytes and contains two 64-bit words. Scalar values
store their bits directly. Strings store a UTF-8 pointer and byte length.
Objects and Lists store pointers to contiguous `Value` arrays. Optional wide
values use an indirect representation defined by the compiler and runtime
together.

Scalar arithmetic stays allocation-free. String literals borrow immutable data
from the executable. Concatenation allocates one exact-size byte buffer. Object
and List copies allocate one exact-size `Value` buffer and copy it directly.

## Allocation and lifetime policy

Adamantium 0.1 uses process-lifetime allocation. Heap storage created for
objects, Lists, concatenated strings, optional wide values, and runtime error
messages remains valid until the process exits. The operating system reclaims
the allocation set at shutdown. There is no tracing garbage collector,
reference counter, manual `free`, or background collector.

This policy is deliberate for the current value semantics. It gives generated
code stable addresses and prevents physical use-after-free and double-free
while aliases, offsets, and lifecycle rules are still evolving. `remove`
invalidates a source binding and runs its language lifecycle hook. It does not
release the underlying allocation.

The tradeoff is that memory use grows with allocations during a long-running
process. Programs that continuously create temporary objects or strings do not
yet return that storage early. A future reclamation implementation must retain
stable addresses for live offsets, preserve value-copy semantics, and pass the
memory-safety regression suite before replacing this policy.

## Error state

Recoverable errors, explicit panics, and package failures use structured
`RuntimeError` values. The runtime stores active `try` depth and the last error
in thread-local state. Successful scalar operations do not allocate error
objects. An unhandled error is rendered once on standard error. A handled error
is converted to an optional string when the matching `try` scope ends.

## Package isolation

Each package call creates a WASI store with explicit limits. Package memory is
limited to 256 MiB, output to 1 MiB, and execution to a fixed fuel budget.
Filesystem access is opt-in. The runtime validates the module before execution
and converts package failures into structured runtime errors.

## Performance rules

Runtime work follows these rules:

- scalar operations and comparisons remain allocation-free;
- immutable string inputs are borrowed instead of copied;
- variable-size outputs allocate once at their final size where possible;
- generated code calls runtime functions directly through the platform ABI;
- WASI engine work is isolated from ordinary arithmetic and printing paths;
- optimization changes must preserve runtime-error source lines and behavior.

Compiler output performance is guarded by the optimization benchmark and CI
regression thresholds. Runtime changes require unit tests for behavior and must
not add allocation to scalar fast paths.

## Shutdown

Returning from `main` uses normal platform shutdown. Explicit `exit()` returns
the requested code immediately. In both cases the operating system reclaims
process-lifetime storage. No Adamantium destructor may depend on a runtime heap
sweep; source-level `__remove__` hooks run according to compiler lifecycle
rules before this boundary.

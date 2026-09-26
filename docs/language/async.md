# Planned async model

Async is a design document, not an implemented Adamantium feature. The words
`async` and `await` are not accepted syntax yet, and programs must not depend on
the model below until the corresponding roadmap items and tests are complete.

## Proposed syntax

An asynchronous function will declare `async fun` and return its normal result
type. Calling it will create a typed task. `await` will suspend the current task
until that result or error is available:

```adamantium
async fun load(path:string) result:string {
    result=await files.read_async(path);
}

async fun main() {
    var text=await load("data.txt");
    print.newline(text);
}
```

This syntax remains provisional and may change before implementation.

## Execution model

- Async functions lower to explicit state machines in MIR.
- A task owns its suspended frame and captured local values.
- `await` is the only implicit suspension point.
- The initial runtime will use cooperative scheduling. It will not promise one
  operating-system thread per task.
- Blocking standard-library operations remain blocking unless their API is
  explicitly asynchronous.
- Dropping the final task handle cancels a task only at a defined cancellation
  point; cleanup hooks run before its frame is released.

## Types, errors, and memory safety

Task results retain their declared Adamantium type. Recoverable errors propagate
through `await` and integrate with `try`; panics keep their source location and
remain distinct from recoverable errors. Aliases and offsets may not outlive a
suspended frame. The compiler must reject borrowed or aliased values crossing an
`await` unless their storage is owned by the task for the complete suspension.

Class lifecycle hooks cannot suspend during the initial implementation. Package
calls are also synchronous until the WASM ABI defines polling, cancellation,
and resource ownership. These restrictions keep async from weakening the
existing memory-safety model.

## Acceptance requirements

Async becomes supported only after parser, type-checker, MIR lowering, runtime,
code-generation, cancellation, error, lifecycle, and cross-platform tests pass.
The implementation must preserve deterministic test execution and report task
leaks or invalid captures as structured compiler diagnostics.

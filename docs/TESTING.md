# Testing Adamantium

Project tests live in `code/tests.ad`, use `#[test]`, and run with `adamantium test run`.

## Assertions

Use `assert(value)` for a boolean condition. A false condition fails the current
test, reports its source line, and prints `expected true, found false`.

```adamantium
#[test]
fun addition_works() {
    assert(2 + 2 == 4);
    assert(4 > 2, "four should be greater than two");
}
```

The optional second argument must be a string and replaces the default failure
message.

## Test file directives

Directives must appear at the beginning of `tests.ad`, before test declarations.

```adamantium
&TestsFile:Parallel[4]
&TestsFile:StopOnFailed:DontStopStarted
```

`Parallel` uses the machine's available parallelism. `Parallel[n]` limits the
number of concurrently running tests to the positive integer `n`. Without a
parallel directive, tests run sequentially.

`StopOnFailed` stops before starting another test after the first failure.
`StopOnFailed:DontStopStarted` also stops scheduling new tests, while tests that
already started in parallel are allowed to finish. Completed results are always
reported in declaration order.

Compiler language tests live under `tests/valid` and `tests/invalid`. Each case contains:

```text
program.ad
expected.txt
expected_exit_code
```

Supporting `.ad` files may be placed beside `program.ad`. Valid cases are compiled and executed, and `expected.txt` is their exact normalized stdout. Invalid cases run compiler analysis and treat every nonempty line in `expected.txt` as a required diagnostic fragment.

Run the suite with:

```text
adamantium test language
cargo test
```

Use `adamantium test language tests --verbose` to show full failure output. Native valid cases require NASM and a supported linker. The Cargo integration test runs on Windows, Linux, and Intel macOS.

## Fuzzing

The `Fuzzing` workflow runs bounded `cargo-fuzz` jobs for the lexer, production
frontend, complete compiler pipeline through code generation, individual code
generators, safe runtime value boundary, and WASM package loader. Pull requests
affecting those components receive a short run, while a
weekly schedule repeats every target. Crashes are uploaded as CI artifacts and
must become deterministic regression tests before the fix is merged.

## Performance regression guard

The `Performance regression` workflow builds and executes the optimization
benchmark on Linux. It records CSV results for `-O0`, `-O1`, and `-O2`, checks
that every measurement is valid, rejects `-O2` assembly larger than `-O0`, and
rejects large execution-time or compilation-time regressions. Results are
uploaded for comparison between runs. The generous runtime margin accounts for
shared GitHub runner noise; detailed local benchmarking remains documented in
the compiler optimization guide.

Memory-safety compile checks have a dedicated cross-platform integration suite:

```text
cargo test -p adamantium-cli --test memory_safety
```

It covers alias and offset lifetimes, removed bindings, invalid dereferences,
escaped offsets, class lifecycle recursion, nested scopes, and regressions for
previously discovered memory-safety bugs. These tests use `adamantium check`
and do not require NASM or a native linker.

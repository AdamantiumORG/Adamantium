# Testing Adamantium

Project tests live in `code/tests.ad`, use `#[test]`, and run with `adamantium test run`.

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

Use `adamantium test language tests --verbose` to show full failure output. Native valid cases require NASM and a supported linker. The Cargo integration test runs on Windows and Linux and is disabled on macOS until a macOS backend exists.

Memory-safety compile checks have a dedicated cross-platform integration suite:

```text
cargo test -p adamantium-cli --test memory_safety
```

It covers alias and offset lifetimes, removed bindings, invalid dereferences,
escaped offsets, class lifecycle recursion, nested scopes, and regressions for
previously discovered memory-safety bugs. These tests use `adamantium check`
and do not require NASM or a native linker.

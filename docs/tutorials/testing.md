# Write and run language tests

Add `code/tests.ad` to a project:

```adamantium
#[test]
fun addition_works() {
    assert(2+2==4);
}

#[test]
fun values_can_be_checked() {
    var value=10;
    assert(value>0,"value should be positive");
}
```

Discover tests without running them:

```text
adamantium test list
```

Run the complete project suite or one named test:

```text
adamantium test run
adamantium test run addition_works
```

A failed assertion produces a nonzero process status suitable for CI. Use
`--verbose` when full test output is needed. Compiler contributors should also
add valid and invalid language regression cases under the repository `tests`
directory. Their format and expected-output rules are documented in
[Testing](../TESTING.md).

# Contributing to Adamantium

Contributions from beginners and experienced developers are welcome. You can
help by fixing bugs, improving diagnostics, writing documentation, or extending
the compiler.

## Set up your environment

Install the Rust toolchain used by the workspace. NASM and a supported native
linker are needed only for tests that build Adamantium executables. Portable
archives carry their own native tools. Run repository commands from the workspace
root, which contains this file and the top-level `Cargo.toml`.

```text
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo run -p adamantium-cli -- check example-project
```

Use `adamantium doctor`, or its Cargo equivalent below, to inspect the native
toolchain available on the current machine:

```text
cargo run -p adamantium-cli -- doctor
```

Continue with the [contributor getting-started guide](docs/compiler/contributor-guide.md).
It maps common changes to crates, explains the compiler pipeline, and lists the
smallest useful test commands for each area. The complete phase contracts are
documented in [compiler architecture](docs/compiler/architecture.md).

## Report a bug

Search existing issues before opening a new one. Include:

- A short description of the problem and the expected behavior.
- The smallest `.ad` example that reproduces it and relevant project settings.
- The exact command and complete error output.
- The commit, operating system, architecture, and tool versions.

Remove private information from examples and logs. Report suspected security
issues through [SECURITY.md](SECURITY.md), not a public issue.

## Propose a feature

Explain the problem and show the intended Adamantium syntax or compiler behavior.
Discuss substantial language changes before implementation. A language change
must account for parsing, semantics, diagnostics, code generation, tests, and
documentation, including Professional Mode where relevant.

## Make changes

- Keep each pull request focused on one bug, feature, or documentation change.
- Write code, diagnostics, tests, and documentation in English.
- Add valid and invalid regression cases for changed language behavior.
- Preserve source spans and structured diagnostics across compiler phases.
- Update examples and contracts when supported behavior changes.
- Keep generated artifacts out of commits and keep `Cargo.lock` tracked.
- Avoid unrelated formatting changes and unnecessary dependencies.

Crate responsibilities and selective CI behavior are documented in
[`docs/compiler/workspace.md`](docs/compiler/workspace.md). The canonical sample
project is [`example-project`](example-project).

## Verify your work

Run the checks relevant to the change:

```text
cargo fmt --all --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
```

For native compiler or runtime changes, also run the ignored native suite on a
supported host with the required tools:

```text
cargo test --locked -p adamantium-cli --test native -- --ignored
cargo run -p adamantium-cli -- build example-project
```

For documentation-only changes, verify wording, relative links, and command
paths. The repository CI runs formatting, spelling, lint, affected-crate tests,
platform CLI tests, release builds, and proof generation.

## Submit a pull request

Describe the problem, resulting behavior, and validation. Link related issues
and state any platform or test limitation. Update [`TODO.md`](TODO.md) when a
change completes a tracked compiler item.

## License

Review [LICENSE.md](LICENSE.md) and the [Code of Conduct](CODE_OF_CONDUCT.md)
before contributing. Submit only material that
you have the right to contribute under the project's license.

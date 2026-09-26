# Versioning and release channels

Adamantium follows Semantic Versioning 2.0.0 for the compiler, CLI, runtime,
standard library contracts, and package ABI identifiers after their first
stable release.

- A patch release fixes behavior without changing supported source or ABI.
- A minor release adds backward-compatible syntax, APIs, targets, or tooling.
- A major release may remove or change stable syntax, APIs, or ABIs.
- Before `1.0.0`, incompatible changes may occur in minor releases and must be
  called out in release notes.

The version in the root `Cargo.toml` is the single source used by every Rust
crate and by `adamantium --version`.

## Channels

| Channel | Git reference | Stability | Purpose |
| --- | --- | --- | --- |
| Stable | `vMAJOR.MINOR.PATCH` | Supported release | Normal users and reproducible builds |
| Development | `dev` | Pre-release | Integration testing before a stable tag |
| Nightly | `nightly` | Continuously replaced | Latest commit that passed the complete CI and CLI tests |

Stable releases are immutable. Development and Nightly references may move.
Release workflows generate notes from merged pull requests and attach checksummed
portable archives. Breaking changes and package ABI changes require an explicit
release-note section.


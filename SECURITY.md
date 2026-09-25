# Security policy

## Supported versions

Adamantium is pre-1.0 software. Security fixes are provided on the current
default branch and in the latest Nightly portable archives. Older commits and
replaced Nightly archives are not maintained.

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Use the repository's
[private security advisory form](https://github.com/AdamantiumORG/Adamantium/security/advisories/new).

Include the affected commit or archive, operating system and architecture, a
minimal reproducer, observed impact, and any known mitigations. Remove secrets
and unrelated personal data from reports. The maintainers will use the private
advisory to coordinate validation, a fix, release notes, and disclosure.

## Security scope

Reports are welcome for the compiler, CLI, runtime, package manager, WASM package
host, generated native programs, build scripts, and portable archives. Examples
include compiler crashes caused by source input, memory-safety violations,
path traversal, package tampering, capability bypasses, command injection, and
unsafe archive contents.

## Current security posture

Adamantium has not completed an independent security audit and should not yet be
used as a security boundary for untrusted native code. Compiler diagnostics and
runtime checks reduce risk but are not a substitute for an audit.

Package checksums verify downloaded bytes against locked metadata. A checksum is
an integrity mechanism, not proof of publisher identity. Community packages are
not controlled by AdamantiumORG or AdmerPRO. Review a package and its declared
capabilities before using it.


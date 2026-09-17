# Creating Adamantium packages

Adamantium packages are WASI Preview 1 command modules published as GitHub Release assets. They may be hosted in any public GitHub repository. Packages owned by `AdmerPRO` or `AdamantiumORG` are official. The CLI displays a warning for other community packages.

## Release contract

Every release needs these assets:

```text
adamantium_packet.wasm
adamantium_packet.toml
SHA256SUMS
adamantium_packet.release.json
```

Version `1.4.2` uses tag `adamantium_packet_1_4_2`. The WASM file must be a WebAssembly 1.0 `wasm32-wasip1` command module with a `_start` entry point.

## Manifest

```toml
[package]
name = "TextTools"
version = "1.4.2"
abi = "wasi-command-v1"
description = "UTF-8 text file helpers"
authors = ["Example Author"]
license = "MIT"
repository = "https://github.com/community/TextTools"

[dependencies]
"https://github.com/community/CoreTools" = "2.0.0"

[permissions]
filesystem = "read-write"

[functions.read_text]
parameters = ["string"]
result = "string"

[functions.exists]
parameters = ["string"]
result = "bool"
command = "file-exists"
```

`package.name` is the identifier used in Adamantium code. It may differ from the repository name. `name`, `version`, and `abi` are required. `description`, `authors`, `license`, and `repository` are optional metadata. Names contain ASCII letters, digits, `_`, or `-`. The current ABI is `wasi-command-v1`.

Versions use an exact, canonical `MAJOR.MINOR.PATCH` value. Version ranges, pre-release labels, build metadata, missing components, and leading zeroes are rejected. The manifest version must equal the requested version.

The optional `[dependencies]` table maps a public GitHub repository URL to an exact version. `adamantium install` resolves all transitive dependencies, rejects cycles and version mismatches, and writes the complete deterministic graph to `adamantium.lock`. Commit this lockfile so every machine and CI job uses the same versions.

`permissions.filesystem` accepts `none`, `read`, or `read-write`. With `none`, the package cannot see the project directory. Both filesystem modes currently receive a preopened project directory from the WASI backend, so `read` is advisory for now. Use `none` when file access is unnecessary.

Each function has a required `parameters` array of at most eight types and a required `result`. `command` defaults to the function name. Supported types are:

```text
i8 i16 i32 i64
u4 u8 u16 u32 u64
f32 f64
string bool None
```

Use `None` only as a result. The ABI does not yet support `f128`, lists, classes, enums, offsets, optional values, or generic types.

## Command protocol

Adamantium starts the module once per function call and supplies:

```text
argv[0] = "adamantium-packet"
argv[1] = command
argv[2..] = function arguments
```

Numbers and booleans use text form. Strings are unchanged. Stdout is the return value: strings keep all bytes, while numeric and boolean results are trimmed and parsed. A `None` result ignores stdout. Write errors to stderr and exit nonzero on failure.

A Rust dispatcher can begin like this:

```rust
use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match (args.next().as_deref(), args.next()) {
        (Some("read_text"), Some(path)) => match fs::read_to_string(path) {
            Ok(text) => { print!("{text}"); ExitCode::SUCCESS }
            Err(error) => { eprintln!("{error}"); ExitCode::FAILURE }
        },
        _ => { eprintln!("unknown command or invalid arguments"); ExitCode::FAILURE }
    }
}
```

Build it with:

```text
rustup target add wasm32-wasip1
cargo test
cargo build --release --target wasm32-wasip1
cp target/wasm32-wasip1/release/text_tools.wasm adamantium_packet.wasm
```

## Publishing

Place the finished WASM module and manifest in the package repository root, then generate a validated release bundle:

```text
adamantium package prepare
```

The command validates the manifest, exact version, ABI and WASM `_start` export. It writes canonical assets to `target/package-release/`. `SHA256SUMS` contains SHA-256 hashes for the WASM module and canonical manifest. `adamantium_packet.release.json` records format version `1`, package identity, ABI, release tag and both hashes.

Publish or update the corresponding GitHub Release with:

```text
gh auth login
adamantium package publish
```

Publishing creates the tag-shaped release when it does not exist. For an existing release it uploads all generated assets with `--clobber`. Run either command with a package directory argument when the package is outside the current directory.

The equivalent manual commands are:

```text
git tag adamantium_packet_1_4_2
git push origin adamantium_packet_1_4_2
gh release create adamantium_packet_1_4_2 target/package-release/* --generate-notes --target HEAD
```

Example release workflow:

```yaml
name: Release Adamantium package
on:
  push:
    tags: ["adamantium_packet_*"]
permissions:
  contents: write
jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-wasip1
      - run: cargo test
      - run: cargo build --release --target wasm32-wasip1
      - run: cp target/wasm32-wasip1/release/text_tools.wasm adamantium_packet.wasm
      - env:
          GH_TOKEN: ${{ github.token }}
        run: gh release create "${{ github.ref_name }}" adamantium_packet.wasm adamantium_packet.toml --generate-notes --verify-tag
```

## Using a package

Add the exact GitHub release version:

```toml
[packages]
"https://github.com/community/TextTools" = "1.4.2"
```

Then install and import it:

```text
adamantium install
```

```adamantium
mod TextTools;
use TextTools:[read_text,exists];

fun main() {
    var text = read_text("notes.txt");
    print.newline(text);
}
```

Single imports such as `use TextTools:read_text;` also work. Without `use`, call `TextTools:read_text("notes.txt")`. Every source file that uses a package declares its own `mod` and `use` statements.

Installed files are stored under `packages/REPOSITORY/VERSION/`. Downloads are cached under `packages/.cache/OWNER/REPOSITORY/VERSION/` and reused after validation. `adamantium check`, `build`, and `run` read transitive packages from `adamantium.lock` and validate the declaration, manifest, ABI, types, version, and WASM header. Run `adamantium install` after changing `requirement.toml`.

## Troubleshooting

- `package module ... is not installed or declared` - check `requirement.toml`, run `adamantium install`, and compare the module with `package.name`.
- `manifest version ... does not match requirement` - publish a manifest matching the requested version.
- `unsupported package ABI` - use `wasi-command-v1`.
- `function ... is not declared` - add `[functions.NAME]` and import the same name.
- `Adamantium package error` - inspect the package's stderr output.

The installer currently validates the downloaded WASM and manifest. Verification against the published checksum file and package signatures remain planned.

## Release checklist

- [ ] The module targets `wasm32-wasip1` and exports `_start`.
- [ ] The manifest describes every public function.
- [ ] Manifest and release versions match.
- [ ] The package requests minimal filesystem access.
- [ ] Package tests pass.
- [ ] The tag uses `adamantium_packet_MAJOR_MINOR_PATCH`.
- [ ] All four generated assets are attached to the release.
- [ ] `SHA256SUMS` matches the published WASM and manifest.
- [ ] A clean project can install, check, build, and run the package.

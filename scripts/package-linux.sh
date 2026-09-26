#!/usr/bin/env bash

set -euo pipefail

compiler_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
distribution_root="$compiler_root/dist"
package_root="$distribution_root/adamantium-linux-x86_64"
archive_path="$distribution_root/adamantium-linux-x86_64.zip"

if [[ "$(uname -s)" != "Linux" || "$(uname -m)" != "x86_64" ]]; then
    echo "Linux portable packages must be built on Linux x86-64." >&2
    exit 1
fi

command -v cargo >/dev/null
command -v nasm >/dev/null
command -v zig >/dev/null
command -v zip >/dev/null

if [[ "${1:-}" != "--skip-build" ]]; then
    cargo rustc --manifest-path "$compiler_root/Cargo.toml" --locked --release \
        -p adamantium-cli --bin adamantium -- -C target-feature=+crt-static
fi

rm -rf "$package_root"
mkdir -p "$package_root/tools/zig"
cp "$compiler_root/target/release/adamantium" "$package_root/adamantium"
cp "$(command -v nasm)" "$package_root/tools/nasm"

zig_executable="$(realpath "$(command -v zig)")"
zig_root="$(dirname "$zig_executable")"
zig_lib="$(zig env | python3 -c 'import json,sys; print(json.load(sys.stdin)["lib_dir"])')"
cp "$zig_executable" "$package_root/tools/zig/zig"
cp -a "$zig_lib" "$package_root/tools/zig/lib"

zig_license=""
for candidate in "$zig_root/LICENSE" "$(dirname "$zig_lib")/LICENSE"; do
    if [[ -f "$candidate" ]]; then
        zig_license="$candidate"
        break
    fi
done
if [[ -z "$zig_license" ]]; then
    echo "Could not find the Zig license file." >&2
    exit 1
fi

cp "$zig_license" "$package_root/ZIG-LICENSE.txt"
cp "$compiler_root/docs/legal/NASM.txt" "$package_root/NASM-LICENSE.txt"
cp "$compiler_root/README.md" "$package_root/README.md"
cat > "$package_root/INSTALL.txt" <<'TEXT'
Adamantium portable for Linux x86-64

1. Extract the complete adamantium-linux-x86_64 directory.
2. Make sure adamantium is executable: chmod +x adamantium
3. Add that directory to PATH.
4. Open a new terminal and run: adamantium --version

Keep the tools directory next to adamantium. Rust, Cargo, NASM, GCC, Clang, and
system development packages are not required to compile Adamantium projects.
TEXT

smoke_root="$distribution_root/portable-linux-smoke-test"
cleanup() {
    rm -rf "$smoke_root"
}
trap cleanup EXIT
rm -rf "$smoke_root"
mkdir -p "$smoke_root/home"

(
    export PATH="$package_root"
    export HOME="$smoke_root/home"
    cd "$smoke_root"
    "$package_root/adamantium" new PortableSmoke
    cd PortableSmoke
    "$package_root/adamantium" build
    ./target/PortableSmoke >/dev/null
)

rm -f "$archive_path" "$archive_path.sha256"
(
    cd "$distribution_root"
    zip -q -r "$(basename "$archive_path")" "$(basename "$package_root")"
)
(
    cd "$distribution_root"
    sha256sum "$(basename "$archive_path")" > "$(basename "$archive_path").sha256"
)
echo "Created $archive_path"

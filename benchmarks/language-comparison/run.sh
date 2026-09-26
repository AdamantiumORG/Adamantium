#!/usr/bin/env bash
set -euo pipefail

iterations="${1:-20}"
if ! [[ "$iterations" =~ ^[1-9][0-9]*$ ]]; then
  echo "iterations must be a positive integer" >&2
  exit 2
fi
for tool in cargo rustc cc; do
  command -v "$tool" >/dev/null || { echo "required benchmark tool '$tool' was not found in PATH" >&2; exit 127; }
done

root="$(cd "$(dirname "$0")" && pwd)"
repository="$(cd "$root/../.." && pwd)"
output="$root/target"
mkdir -p "$output"
cd "$repository"

cargo build --release --locked -p adamantium-cli
adamantium="$repository/target/release/adamantium"
adamantium_project="$root/adamantium"
adamantium_exe="$adamantium_project/target/LanguageComparison"
c_exe="$output/comparison-c"
rust_exe="$output/comparison-rust"

printf 'language,build_ms,executable_bytes,average_run_ms\n'

benchmark() {
  local language="$1"
  local executable="$2"
  shift 2
  local start end build_ns run_start run_end actual size
  start="$(date +%s%N)"
  "$@" >/dev/null
  end="$(date +%s%N)"
  build_ns="$((end - start))"
  actual="$($executable)"
  if [[ "$actual" != "6" ]]; then
    echo "$language returned unexpected output '$actual'" >&2
    exit 1
  fi
  run_start="$(date +%s%N)"
  for ((index = 0; index < iterations; ++index)); do
    "$executable" >/dev/null
  done
  run_end="$(date +%s%N)"
  size="$(wc -c < "$executable")"
  awk -v language="$language" -v build="$build_ns" -v size="$size" \
    -v run="$((run_end - run_start))" -v count="$iterations" \
    'BEGIN { printf "%s,%.3f,%d,%.3f\n", language, build / 1000000, size, run / count / 1000000 }'
}

benchmark Adamantium "$adamantium_exe" "$adamantium" build "$adamantium_project" -O2
benchmark C "$c_exe" cc -O2 "$root/c/main.c" -o "$c_exe"
benchmark Rust "$rust_exe" rustc -C opt-level=3 "$root/rust/main.rs" -o "$rust_exe"

#!/usr/bin/env bash
set -euo pipefail

project="${1:-benchmarks/optimization}"
iterations="${2:-20}"
root="$(cd "$project" && pwd)"
name="$(sed -n '/^name[[:space:]]*=/s/^[^\"]*\"\([^\"]*\)\".*/\1/p' "$root/project.toml" | head -n1)"
cli="$(cd "$(dirname "$0")/.." && pwd)/target/release/adamantium"
cargo build --release -p adamantium-cli

printf 'level,compile_ms,assembly_bytes,executable_bytes,average_run_ms\n'
for level in -O0 -O1 -O2; do
  start="$(date +%s%N)"
  "$cli" build "$root" "$level" >/dev/null
  end="$(date +%s%N)"
  executable="$root/target/$name"
  assembly="$root/target/$name.asm"
  run_start="$(date +%s%N)"
  for ((index=0; index<iterations; index++)); do timeout 30s "$executable" >/dev/null; done
  run_end="$(date +%s%N)"
  awk -v level="$level" -v compile="$((end-start))" -v asm="$(wc -c < "$assembly")" -v exe="$(wc -c < "$executable")" -v run="$((run_end-run_start))" -v count="$iterations" 'BEGIN { printf "%s,%.3f,%d,%d,%.3f\n", level, compile/1000000, asm, exe, run/count/1000000 }'
done

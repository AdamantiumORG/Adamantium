#!/usr/bin/env python3
import csv
import pathlib
import sys

path = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "performance.csv")
with path.open(newline="", encoding="utf-8") as handle:
    rows = {row["level"]: row for row in csv.DictReader(handle)}

missing = {"-O0", "-O1", "-O2"} - rows.keys()
if missing:
    raise SystemExit(f"missing benchmark rows: {', '.join(sorted(missing))}")

for level, row in rows.items():
    for field in ("compile_ms", "assembly_bytes", "executable_bytes", "average_run_ms"):
        value = float(row[field])
        if value <= 0:
            raise SystemExit(f"{level} produced invalid {field}: {value}")

o0 = rows["-O0"]
o2 = rows["-O2"]
if int(o2["assembly_bytes"]) > int(o0["assembly_bytes"]):
    raise SystemExit("-O2 assembly is larger than -O0 assembly")
if float(o2["average_run_ms"]) > max(float(o0["average_run_ms"]) * 2.0, 5.0):
    raise SystemExit("-O2 execution time regressed beyond the allowed noise margin")
if float(o2["compile_ms"]) > 120_000:
    raise SystemExit("-O2 compilation exceeded the 120 second guard")

print("performance regression guard passed")

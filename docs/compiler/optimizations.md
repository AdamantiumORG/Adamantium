# Optimization levels

Adamantium applies optimizations after semantic analysis and before NASM code generation.

| Level | Behavior |
| --- | --- |
| `-O0` | Preserve typed IR and generated assembly for debugging. |
| `-O1` | Default. Fold constants, propagate scalar constants, simplify expressions and constant branches, and remove unreachable statements. |
| `-O2` | Apply `-O1`, remove unused local assignments and unreachable functions, preserve side effects, and remove redundant assembly instructions. |

Use one level with `adamantium build` or `adamantium run`:

```text
adamantium build -O0
adamantium build ./project -O2
adamantium run -O1 --name value
```

If no option is supplied, Adamantium uses `-O1`. `adamantium check` performs semantic analysis only, so optimization levels do not apply to it.

At `-O2`, the optimizer also inlines small, parameterless functions whose body
has no local state or control flow when they are called as standalone
statements. The deliberately narrow eligibility rule prevents slot capture,
recursive expansion, and changes to observable error or lifecycle behavior.

The optimizer never folds an operation that would report overflow, division by zero, an invalid conversion, or another runtime error. It keeps effectful calls when their assigned value is unused. Function elimination starts at the selected executable entry point and follows ordinary calls, constructors, methods, and lifecycle hooks.

## Benchmarks

Run the platform script against an Adamantium project:

```text
./scripts/benchmark-optimizations.ps1 ./benchmarks/optimization 20
./scripts/benchmark-optimizations.sh ./benchmarks/optimization 20
```

Each script reports compiler wall time, assembly size, executable size, and average program wall time for `-O0`, `-O1`, and `-O2`. Run benchmarks on an otherwise idle machine and compare results from the same commit and toolchain.

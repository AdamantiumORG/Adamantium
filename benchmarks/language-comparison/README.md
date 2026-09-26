# Adamantium, C, and Rust comparison benchmark

This benchmark measures one deliberately small integer workload implemented in
Adamantium, C, and Rust. It is a reproducible harness for tracking native-code
performance, not a general ranking of the languages.

All three programs execute the same loop, print the result so the computation
remains observable, and use optimized release builds. The scripts report build
time, executable size, and average execution time as CSV.

Windows:

```powershell
./benchmarks/language-comparison/run.ps1 -Iterations 20
```

Linux:

```text
./benchmarks/language-comparison/run.sh 20
```

Requirements are the Adamantium native toolchain, `rustc`, and `clang` on
Windows or a `cc`-compatible C compiler on Linux.
Run on an otherwise idle machine, record the commit, operating system, CPU,
compiler versions, and use several runs. Do not publish a single measurement as
a broad language-performance claim.

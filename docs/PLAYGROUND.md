# Try Adamantium online

GitHub Codespaces provides a browser editor and terminal with the Adamantium
compiler dependencies preconfigured. It does not require Rust, NASM, or a
linker to be installed on the local computer.

[Open Adamantium in GitHub Codespaces](https://codespaces.new/AdamantiumORG/Adamantium?quickstart=1)

After the container finishes its setup, run:

```text
adamantium check example-project
adamantium run example-project
```

The first container creation builds the CLI and can take several minutes.
Codespaces usage is subject to the GitHub account's available quota. This is a
complete cloud development environment rather than a sandboxed public REPL;
the planned browser playground can later reuse the compiler frontend through a
dedicated WebAssembly interface.

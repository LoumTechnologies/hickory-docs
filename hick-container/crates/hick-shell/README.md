# hick-shell

Lightweight POSIX/bash shell interpreter for `<hick:script>` elements.

A cheaper alternative to the full container-based `<hick:exec>` model. Parses
shell scripts using the `brush-parser` crate, resolves external commands to
individual WASI `.wasm` binaries found in a toolchain directory (via
`wasmtime`), and handles builtins (echo, true/false, export, etc.) in pure Rust.

Entry point: `run_script()` returns a `ScriptOutput` containing captured
stdout, stderr, and exit code.

Intended for simple shell steps that don't need a full isolated container
environment.

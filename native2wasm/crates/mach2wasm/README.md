# mach2wasm

AOT transpiler for macOS Mach-O x86_64 binaries.

Structurally identical to `elf2wasm` and `pe2wasm`. Parses Mach-O files via
`goblin` and embeds Darwin-syscall shims (`darwin_syscall.h`, etc.).

Warns when the binary links dylibs — complex dynamic macOS binaries are expected
to fall through to the VM strategy (using Darling) for full support.

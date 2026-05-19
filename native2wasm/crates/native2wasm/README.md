# native2wasm

Top-level CLI binary and strategy-dispatch library for converting native
binaries to WASM.

Detects the input binary's format (ELF/Mach-O/PE), CPU architecture, and OS
using `goblin`, then resolves the `--strategy` flag and dispatches accordingly:

| Strategy | Path |
|---|---|
| `aot` | Format-specific crate → `native2wasm-core` LLVM pipeline |
| `emu` | `emu-x86` or `emu-arm64` emulator provider |
| `vm` | Docker or no-Docker OCI container image → `c2w` |
| `auto` | Picks the best strategy for the detected format and arch |

Optional overrides: Remill semantics dir, WASI sysroot, emulator WASM path,
optimization level, `--emit-llvm`.

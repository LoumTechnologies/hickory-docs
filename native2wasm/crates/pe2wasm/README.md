# pe2wasm

AOT transpiler for Windows PE x86_64 binaries.

Structurally identical to `elf2wasm`. Parses PE files via `goblin` and embeds
NT-syscall shims (`nt_syscall.h`, `syscall_dispatch.c`, `shim_io.c`,
`shim_memory.c`, `shim_misc.c`).

Warns when the input imports DLLs (dynamic PE) — complex DLL-dependent binaries
require the EMU or VM (Wine) strategy for full support.

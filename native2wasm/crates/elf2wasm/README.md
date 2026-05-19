# elf2wasm

AOT transpiler for Linux ELF x86_64 binaries.

Parses ELF files via `goblin`, implements `ParsedBinary`, and embeds
Linux-specific syscall shims (`linux_syscall.h`, `syscall_dispatch.c`,
`shim_io.c`, `shim_memory.c`, `shim_misc.c`) as static byte slices compiled
into the library.

Also exposes a `transpile(input, output, opt_level)` convenience API.

Warns (but does not abort) when the input is dynamically linked — dynamic
binaries are only partially supported and may require the EMU or VM strategy.

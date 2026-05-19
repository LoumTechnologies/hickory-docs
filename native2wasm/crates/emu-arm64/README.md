# emu-arm64

Implements `EmulatorProvider` for ARM64 Linux ELF binaries using QEMU TCI
(`qemu-aarch64.wasm`).

`QemuTciProvider` checks an explicit path, then
`~/.cache/native2wasm/emulators/qemu-tci-aarch64/emulator.wasm`, and emits
detailed build instructions if neither is found.

Guest filesystem preparation copies the binary to `/guest/binary` and creates
`/guest/lib` for potential dynamic linker shims.

This is the only supported ARM64 emulation path. macOS and Windows ARM64
binaries require the VM strategy.

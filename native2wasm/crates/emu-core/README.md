# emu-core

Framework crate providing the `EmulatorProvider` trait and shared emulation
pipeline.

## EmulatorProvider trait

Defines how a concrete emulator:
- Locates its pre-compiled WASM binary (`emulator_wasm()`)
- Prepares a guest filesystem with the target binary (`prepare_guest_fs()`)
- Specifies emulator arguments baked into the module entry point

## Shared pipeline

`run_emulation` uses `wasi-vfs` to pack the guest filesystem into the final
WASM output, producing a self-contained module that boots directly into the
emulated binary.

## Supporting modules

- `cache` — lookup and storage in `~/.cache/native2wasm/emulators/`
- `loader` — locates prebuilt loader shims

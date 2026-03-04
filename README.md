# native2wasm

Any native binary to WASI WebAssembly.

native2wasm converts x86_64 and ARM64 executables (Linux ELF, macOS Mach-O, Windows PE) into portable WASI WebAssembly modules. It selects from three strategies depending on the input binary:

| Strategy | When | How | Speed |
|----------|------|-----|-------|
| **AOT** | Static binaries | Remill lift to LLVM IR, compile to wasm32 | Fastest |
| **EMU** | Dynamic binaries | User-mode emulator (blink/QEMU TCI) packed via wasi-vfs | Fast |
| **VM** | Universal fallback | Full Linux VM via container2wasm | Slowest |

## Coverage Matrix

| Input | x86_64 | ARM64 |
|-------|--------|-------|
| Linux ELF (static) | AOT | AOT |
| Linux ELF (dynamic) | EMU (blink) | EMU (QEMU TCI) |
| macOS Mach-O | EMU (blink + loader) | VM (Darling) |
| Windows PE | EMU (blink + loader) | VM (Wine) |

## Installation

```bash
cargo install --path crates/native2wasm
```

## Usage

```bash
# Auto-select strategy (recommended)
native2wasm ./hello -o hello.wasm

# Force a specific strategy
native2wasm ./hello -o hello.wasm --strategy aot
native2wasm ./hello -o hello.wasm --strategy emu
native2wasm ./hello -o hello.wasm --strategy vm

# Use a custom emulator binary
native2wasm ./hello -o hello.wasm --strategy emu --emulator-path ./blink.wasm

# Emit LLVM IR instead of WASM (AOT only)
native2wasm ./hello -o hello.ll --strategy aot --emit-llvm

# Verbose output
native2wasm ./hello -o hello.wasm -v
```

### Auto-Selection Logic

1. Static binary -> **AOT** (Remill lift, zero overhead)
2. Dynamic binary + emulator available -> **EMU** (user-mode emulator)
3. Otherwise -> **VM** (full Linux VM in WASM)

## Building Emulator Artifacts

The EMU strategy requires pre-compiled emulator WASM binaries. Build scripts are provided:

### blink (x86_64 emulation)

```bash
# Requires wasi-sdk at $WASI_SDK_PATH or /opt/wasi-sdk
scripts/build-blink.sh
```

### QEMU TCI (ARM64 emulation, experimental)

```bash
# Requires wasi-sdk and meson
scripts/build-qemu-tci.sh
```

### Loader shims (macOS/Windows emulation)

```bash
# Requires x86_64-linux-musl-gcc
scripts/build-loaders.sh
```

All artifacts are cached at `~/.cache/native2wasm/`.

## AOT Prerequisites

The AOT strategy requires:
- [Remill](https://github.com/lifting-bits/remill) (installed or built from source)
- `clang` (for compiling syscall shims to wasm32-wasi)
- `wasm-ld` (LLVM linker for wasm32)
- `llc` (LLVM static compiler)

## Crate Architecture

```
crates/
  native2wasm/        Unified CLI entry point + strategy dispatch
  native2wasm-core/   Shared AOT pipeline (Remill lift, link, shim compile)
  elf2wasm/           Linux ELF x86_64 AOT transpiler + syscall shims
  mach2wasm/          macOS Mach-O x86_64 AOT transpiler + syscall shims
  pe2wasm/            Windows PE x86_64 AOT transpiler + syscall shims
  emu-core/           Emulation framework (provider trait, pipeline, cache)
  emu-x86/            blink providers (Linux, macOS, Windows x86_64)
  emu-arm64/          QEMU TCI provider (Linux ARM64)
  c2w/                Container-to-WASM (VM strategy)
  oci-pull/           OCI registry client for pulling container images
```

## Development

```bash
# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Check formatting and lints
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings

# Run end-to-end tests (requires emulator artifacts)
cargo test --workspace -- --ignored
```

## License

See individual crate licenses.

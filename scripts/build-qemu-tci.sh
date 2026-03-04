#!/usr/bin/env bash
# Build QEMU TCI (ARM64 user-mode emulator) as a WASI WebAssembly module.
#
# EXPERIMENTAL: QEMU WASM support is immature. This script may require
# adjustments depending on your QEMU fork and wasi-sdk version.
#
# Requires: wasi-sdk at $WASI_SDK_PATH or /opt/wasi-sdk, meson, ninja
# Output:   ~/.cache/native2wasm/emulators/qemu-tci-aarch64/emulator.wasm

set -euo pipefail

WASI_SDK="${WASI_SDK_PATH:-/opt/wasi-sdk}"
CACHE_DIR="${HOME}/.cache/native2wasm/emulators/qemu-tci-aarch64"
BUILD_DIR="${TMPDIR:-/tmp}/native2wasm-build-qemu-tci"

if [ ! -d "$WASI_SDK" ]; then
    echo "Error: wasi-sdk not found at $WASI_SDK"
    echo "Set \$WASI_SDK_PATH or install to /opt/wasi-sdk"
    exit 1
fi

for tool in meson ninja; do
    if ! command -v "$tool" &>/dev/null; then
        echo "Error: $tool not found. Install via your package manager."
        exit 1
    fi
done

echo "==> [EXPERIMENTAL] Building QEMU TCI for wasm32-wasi"
echo "==> Using wasi-sdk at $WASI_SDK"
echo "==> Build directory: $BUILD_DIR"

rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

echo "==> Cloning qemu-wasm fork..."
git clone --depth 1 https://github.com/nicktehrany/qemu-wasm.git
cd qemu-wasm

# Create meson cross-compilation file for wasm32-wasi
cat > wasm32-wasi-cross.txt <<CROSS
[binaries]
c = '${WASI_SDK}/bin/clang'
cpp = '${WASI_SDK}/bin/clang++'
ar = '${WASI_SDK}/bin/ar'
ranlib = '${WASI_SDK}/bin/ranlib'
strip = '${WASI_SDK}/bin/strip'

[built-in options]
c_args = ['--sysroot=${WASI_SDK}/share/wasi-sysroot', '-D_WASI_EMULATED_SIGNAL', '-D_WASI_EMULATED_PROCESS_CLOCKS']
c_link_args = ['--sysroot=${WASI_SDK}/share/wasi-sysroot', '-lwasi-emulated-signal', '-lwasi-emulated-process-clocks']
cpp_args = ['--sysroot=${WASI_SDK}/share/wasi-sysroot']
cpp_link_args = ['--sysroot=${WASI_SDK}/share/wasi-sysroot']

[host_machine]
system = 'wasi'
cpu_family = 'wasm32'
cpu = 'wasm32'
endian = 'little'
CROSS

echo "==> Configuring QEMU with TCI interpreter..."
meson setup build \
    --cross-file wasm32-wasi-cross.txt \
    -Dtarget_list=aarch64-linux-user \
    -Dtcg_interpreter=true \
    -Ddefault_targets='' \
    -Dsystem=false \
    -Dtools=false \
    -Dguest_agent=false \
    -Ddocs=false \
    -Dtests=false

echo "==> Building QEMU TCI..."
ninja -C build qemu-aarch64

echo "==> Installing to cache..."
mkdir -p "$CACHE_DIR"
cp build/qemu-aarch64 "$CACHE_DIR/emulator.wasm"

echo "==> Done: $CACHE_DIR/emulator.wasm"
echo "    Size: $(wc -c < "$CACHE_DIR/emulator.wasm" | tr -d ' ') bytes"
echo ""
echo "NOTE: QEMU WASM support is experimental. If this build fails,"
echo "try upstream QEMU 10.1+ with --enable-tcg-interpreter."

rm -rf "$BUILD_DIR"

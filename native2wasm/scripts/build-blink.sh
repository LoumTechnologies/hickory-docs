#!/usr/bin/env bash
# Build blink (x86_64 user-mode emulator) as a WASI WebAssembly module.
#
# Requires: wasi-sdk at $WASI_SDK_PATH or /opt/wasi-sdk
# Output:   ~/.cache/native2wasm/emulators/blink/emulator.wasm

set -euo pipefail

WASI_SDK="${WASI_SDK_PATH:-/opt/wasi-sdk}"
CACHE_DIR="${HOME}/.cache/native2wasm/emulators/blink"
BUILD_DIR="${TMPDIR:-/tmp}/native2wasm-build-blink"

if [ ! -d "$WASI_SDK" ]; then
    echo "Error: wasi-sdk not found at $WASI_SDK"
    echo "Set \$WASI_SDK_PATH or install to /opt/wasi-sdk"
    echo "See: https://github.com/WebAssembly/wasi-sdk/releases"
    exit 1
fi

echo "==> Using wasi-sdk at $WASI_SDK"
echo "==> Build directory: $BUILD_DIR"

rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

echo "==> Cloning blink..."
git clone --depth 1 https://github.com/jart/blink.git
cd blink

echo "==> Configuring blink for wasm32-wasi..."
CC="${WASI_SDK}/bin/clang" \
AR="${WASI_SDK}/bin/ar" \
CFLAGS="--sysroot=${WASI_SDK}/share/wasi-sysroot" \
./configure \
    --target=wasm32-wasi \
    --disable-jit \
    --disable-fork \
    --disable-signals

echo "==> Building blink..."
make -j"$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)"

echo "==> Installing to cache..."
mkdir -p "$CACHE_DIR"
cp blink "$CACHE_DIR/emulator.wasm"

echo "==> Done: $CACHE_DIR/emulator.wasm"
echo "    Size: $(wc -c < "$CACHE_DIR/emulator.wasm" | tr -d ' ') bytes"

rm -rf "$BUILD_DIR"

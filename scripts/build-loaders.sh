#!/usr/bin/env bash
# Build Mach-O and PE loader shims as static Linux x86_64 ELFs.
#
# These loaders run inside blink to load non-Linux binaries:
#   - macho_loader: loads Mach-O binaries, patches XNU syscalls to Linux
#   - pe_loader: loads PE binaries, resolves IAT to Linux syscall thunks
#
# Requires: x86_64-linux-musl-gcc (musl cross-compiler)
# Output:   ~/.cache/native2wasm/loaders/{macho_loader,pe_loader}

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SHIMS_DIR="${SCRIPT_DIR}/../crates/emu-x86/shims"
CACHE_DIR="${HOME}/.cache/native2wasm/loaders"

if ! command -v x86_64-linux-musl-gcc &>/dev/null; then
    echo "Error: x86_64-linux-musl-gcc not found."
    echo "Install musl cross-compiler:"
    echo "  macOS:  brew install FiloSottile/musl-cross/musl-cross"
    echo "  Linux:  apt install musl-tools gcc-x86-64-linux-gnu"
    exit 1
fi

CC=x86_64-linux-musl-gcc
CFLAGS="-static -Os -Wall -Wextra -Werror -std=c11"

mkdir -p "$CACHE_DIR"

echo "==> Building Mach-O loader..."
$CC $CFLAGS -o "$CACHE_DIR/macho_loader" "$SHIMS_DIR/macho_loader.c"
echo "    Output: $CACHE_DIR/macho_loader ($(wc -c < "$CACHE_DIR/macho_loader" | tr -d ' ') bytes)"

echo "==> Building PE loader..."
$CC $CFLAGS -o "$CACHE_DIR/pe_loader" "$SHIMS_DIR/pe_loader.c"
echo "    Output: $CACHE_DIR/pe_loader ($(wc -c < "$CACHE_DIR/pe_loader" | tr -d ' ') bytes)"

echo "==> Done. Loaders cached at $CACHE_DIR/"

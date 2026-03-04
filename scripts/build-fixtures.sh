#!/usr/bin/env bash
# Build test fixtures: minimal C programs compiled to static ELFs.
#
# Requires: x86_64-linux-musl-gcc, aarch64-linux-musl-gcc
# Output:   tests/fixtures/hello_x86_64.elf, tests/fixtures/hello_arm64.elf

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FIXTURES_DIR="${SCRIPT_DIR}/../tests/fixtures"
SRC_DIR="${FIXTURES_DIR}/src"

mkdir -p "$FIXTURES_DIR"

CFLAGS="-static -Os -nostdlib -Wall -Wextra -Werror"

echo "==> Building x86_64 fixture..."
if command -v x86_64-linux-musl-gcc &>/dev/null; then
    x86_64-linux-musl-gcc $CFLAGS -o "$FIXTURES_DIR/hello_x86_64.elf" "$SRC_DIR/hello.c"
    echo "    Output: $FIXTURES_DIR/hello_x86_64.elf"
else
    echo "    SKIP: x86_64-linux-musl-gcc not found"
fi

echo "==> Building ARM64 fixture..."
if command -v aarch64-linux-musl-gcc &>/dev/null; then
    aarch64-linux-musl-gcc $CFLAGS -o "$FIXTURES_DIR/hello_arm64.elf" "$SRC_DIR/hello.c"
    echo "    Output: $FIXTURES_DIR/hello_arm64.elf"
else
    echo "    SKIP: aarch64-linux-musl-gcc not found"
fi

echo "==> Done."

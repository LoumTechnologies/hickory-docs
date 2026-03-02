#!/usr/bin/env bash
# Remill + wasi-sdk toolchain environment for mach2wasm / pe2wasm pipeline.
# Usage: source scripts/setup-toolchain.sh

export REMILL_PREFIX=/Volumes/Yellow-4TB/mac/remill
export REMILL_DEPS_PREFIX=/Volumes/Yellow-4TB/mac/remill-deps
export REMILL_SEMANTICS_DIR="$REMILL_PREFIX/share/remill/semantics"
export WASI_SDK=/Volumes/Yellow-4TB/mac/wasi-sdk-25.0-x86_64-macos
export WASI_SYSROOT="$WASI_SDK/share/wasi-sysroot"
export PATH="$WASI_SDK/bin:$PATH"

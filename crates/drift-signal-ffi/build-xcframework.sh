#!/usr/bin/env bash
#
# Build drift-signal-ffi as an XCFramework for iOS (device + simulator) and macOS.
#
# Prerequisites:
#   rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-darwin x86_64-apple-darwin
#
# Usage:
#   ./build-xcframework.sh [--release]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

PROFILE="debug"
CARGO_FLAGS=""
if [[ "${1:-}" == "--release" ]]; then
    PROFILE="release"
    CARGO_FLAGS="--release"
fi

LIB_NAME="libdrift_signal_ffi"

echo "==> Building for aarch64-apple-ios (device)..."
cargo build $CARGO_FLAGS --target aarch64-apple-ios

echo "==> Building for aarch64-apple-ios-sim (simulator arm64)..."
cargo build $CARGO_FLAGS --target aarch64-apple-ios-sim

echo "==> Building for x86_64-apple-ios (simulator x86_64)..."
cargo build $CARGO_FLAGS --target x86_64-apple-ios

echo "==> Building for aarch64-apple-darwin (macOS arm64)..."
cargo build $CARGO_FLAGS --target aarch64-apple-darwin

echo "==> Building for x86_64-apple-darwin (macOS x86_64)..."
cargo build $CARGO_FLAGS --target x86_64-apple-darwin

TARGET_DIR="../../target"

echo "==> Creating fat simulator library..."
mkdir -p "$TARGET_DIR/ios-simulator-fat/$PROFILE"
lipo -create \
    "$TARGET_DIR/aarch64-apple-ios-sim/$PROFILE/$LIB_NAME.a" \
    "$TARGET_DIR/x86_64-apple-ios/$PROFILE/$LIB_NAME.a" \
    -output "$TARGET_DIR/ios-simulator-fat/$PROFILE/$LIB_NAME.a"

echo "==> Creating fat macOS library..."
mkdir -p "$TARGET_DIR/macos-fat/$PROFILE"
lipo -create \
    "$TARGET_DIR/aarch64-apple-darwin/$PROFILE/$LIB_NAME.a" \
    "$TARGET_DIR/x86_64-apple-darwin/$PROFILE/$LIB_NAME.a" \
    -output "$TARGET_DIR/macos-fat/$PROFILE/$LIB_NAME.a"

echo "==> Generating Swift bindings..."
cargo run --bin uniffi-bindgen generate \
    --library "$TARGET_DIR/aarch64-apple-darwin/$PROFILE/$LIB_NAME.dylib" \
    --language swift \
    --out-dir "$SCRIPT_DIR/generated"

SWIFT_FILE=$(find "$SCRIPT_DIR/generated" -name "*.swift" | head -1)
HEADER_DIR="$SCRIPT_DIR/generated/headers"
mkdir -p "$HEADER_DIR"

# Move the generated modulemap + header into the headers dir
mv "$SCRIPT_DIR/generated/"*.modulemap "$HEADER_DIR/module.modulemap" 2>/dev/null || true
mv "$SCRIPT_DIR/generated/"*.h "$HEADER_DIR/" 2>/dev/null || true

OUT_DIR="$SCRIPT_DIR/DriftSignalFFI.xcframework"
rm -rf "$OUT_DIR"

echo "==> Building XCFramework..."
xcodebuild -create-xcframework \
    -library "$TARGET_DIR/aarch64-apple-ios/$PROFILE/$LIB_NAME.a" \
    -headers "$HEADER_DIR" \
    -library "$TARGET_DIR/ios-simulator-fat/$PROFILE/$LIB_NAME.a" \
    -headers "$HEADER_DIR" \
    -library "$TARGET_DIR/macos-fat/$PROFILE/$LIB_NAME.a" \
    -headers "$HEADER_DIR" \
    -output "$OUT_DIR"

echo ""
echo "Done! Outputs:"
echo "  XCFramework: $OUT_DIR"
echo "  Swift file:  $SWIFT_FILE"
echo ""
echo "Add both the .xcframework and the .swift file to your Xcode project."

#!/usr/bin/env bash
# Build one downloadable `hickory` artifact.
#
# Usage: scripts/dist.sh <target-triple> [version]
#
# Called identically by `just dist` and by the release workflows, so what a
# maintainer can reproduce locally is byte-for-byte what CI ships (same as
# scripts/check-codegen.sh serving both the hook and CI).
#
# Output, under dist/:
#   hickory-<version>-<target>.tar.gz   (unix)  or  .zip (windows)
#   hickory-<version>-<target>.<ext>.sha256
#
# The archive holds the binary plus README, LICENCE, and examples/, so a
# stranger who downloads it can run `hickory test examples/` immediately —
# that is the acceptance test for this whole path (see
# docs/guarantees/release/a-download-runs-without-a-rust-toolchain.md).
set -euo pipefail
cd "$(dirname "$0")/.."

TARGET="${1:?usage: dist.sh <target-triple> [version]}"
VERSION="${2:-$(sed -n 's/^version = "\(.*\)"$/\1/p' crates/hickory-cli/Cargo.toml | head -1)}"

if [ -z "$VERSION" ]; then
  echo "::error::could not determine a version, and none was passed." >&2
  echo "Pass one explicitly:  scripts/dist.sh $TARGET 0.1.0" >&2
  exit 1
fi

# `hickory --version` should report the version of the artifact somebody
# downloaded, not the workspace's stale Cargo.toml number. main.rs prefers
# this over CARGO_PKG_VERSION when it is set at compile time.
export HICKORY_VERSION="$VERSION"

# The CLI is not pure Rust, however much it looks it: `ring` (via rustls),
# `libsodium-sys` (via macaroon, via hick-token), `zstd-sys`, `bzip2-sys`, and
# `lzma-sys` all compile C, and libsodium's build is autotools rather than the
# `cc` crate. That is why the release matrix builds every Linux target on a
# runner of its own architecture instead of cross-compiling: a cross build has
# to get a C compiler, an archiver, and an autotools `--host` triple right for
# five separate build scripts, and each one fails differently. macOS is the
# exception, because Apple's toolchain cross-compiles between its own two
# architectures as a first-class case and `libsodium-sys` special-cases it.
case "$TARGET" in
  *-apple-darwin)
    # The oldest macOS the artifact promises to run on. Without this, the
    # binary silently inherits whatever the runner image happens to be.
    export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}"
    ;;
esac

echo "==> building hickory $VERSION for $TARGET"
cargo build --release --locked -p hickory-cli --target "$TARGET"

BIN=hickory
EXT=tar.gz
case "$TARGET" in
  *-windows-*)
    BIN=hickory.exe
    EXT=zip
    ;;
esac

BUILT="target/$TARGET/release/$BIN"
if [ ! -f "$BUILT" ]; then
  echo "::error::cargo reported success but $BUILT does not exist." >&2
  echo "The [[bin]] name in crates/hickory-cli/Cargo.toml is what this path" >&2
  echo "is derived from; if it was renamed, update scripts/dist.sh to match." >&2
  exit 1
fi

NAME="hickory-$VERSION-$TARGET"
STAGE="dist/$NAME"
rm -rf "$STAGE"
mkdir -p "$STAGE"

cp "$BUILT" "$STAGE/$BIN"
cp README.md LICENSE "$STAGE/"
cp -R examples "$STAGE/examples"
# .hick-cache is a local execution artifact, never part of a download.
rm -rf "$STAGE/examples/.hick-cache"

ARCHIVE="dist/$NAME.$EXT"
rm -f "$ARCHIVE"
case "$EXT" in
  tar.gz)
    # --sort/--mtime/--owner keep the tarball reproducible: two builds of the
    # same commit should differ only where the compiler made them differ.
    tar -C dist \
      --sort=name --mtime="@0" --owner=0 --group=0 --numeric-owner \
      -czf "$ARCHIVE" "$NAME" 2>/dev/null ||
      tar -C dist -czf "$ARCHIVE" "$NAME"
    ;;
  zip)
    if command -v 7z >/dev/null 2>&1; then
      (cd dist && 7z a -bso0 -bsp0 "$NAME.zip" "$NAME" >/dev/null)
    elif command -v zip >/dev/null 2>&1; then
      (cd dist && zip -qr "$NAME.zip" "$NAME")
    elif command -v powershell >/dev/null 2>&1; then
      powershell -NoProfile -Command \
        "Compress-Archive -Path 'dist/$NAME' -DestinationPath 'dist/$NAME.zip' -Force"
    else
      echo "::error::no zip tool found (tried 7z, zip, powershell)." >&2
      echo "Install one, or build a non-Windows target instead." >&2
      exit 1
    fi
    ;;
esac

# Checksums travel with the artifact so the installer can refuse a corrupted
# or substituted download.
if command -v sha256sum >/dev/null 2>&1; then
  (cd dist && sha256sum "$NAME.$EXT" > "$NAME.$EXT.sha256")
elif command -v shasum >/dev/null 2>&1; then
  (cd dist && shasum -a 256 "$NAME.$EXT" > "$NAME.$EXT.sha256")
else
  echo "::error::no sha256 tool found (tried sha256sum, shasum)." >&2
  exit 1
fi

rm -rf "$STAGE"
echo "==> $ARCHIVE"
cat "dist/$NAME.$EXT.sha256"

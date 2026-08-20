#!/usr/bin/env bash
# Build the downloadable Hickory Docs desktop app for one platform.
#
# Usage: scripts/dist-desktop.sh [version] [target-triple]
#
# The desktop app is a SEPARATE download from the CLI, with its own installer
# shape per platform, because that is what each platform's users expect: a
# `.dmg` you drag, an `.msi` you double-click, a `.deb`/`.AppImage` you install
# or run. Cramming a GUI application into the CLI's tarball would mean a macOS
# user hand-assembling a `.app`, and a CLI user downloading a webview.
#
# The two share one engine (`hickory-cli::serve` runs in-process inside the
# app), so they are one product with two front doors — see
# docs/specs/freeform/local-only.md.
#
# Output, under dist/desktop/: whatever bundles this platform produces, each
# with a `.sha256` beside it.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION="${1:-$(sed -n 's/^version = "\(.*\)"$/\1/p' apps/desktop/src-tauri/Cargo.toml | head -1)}"
HOST_TARGET="$(rustc -vV | sed -n 's/^host: //p')"
TARGET="${2:-${TARGET:-$HOST_TARGET}}"
if [ -z "$VERSION" ]; then
  echo "::error::could not determine a version, and none was passed." >&2
  echo "Pass one explicitly:  scripts/dist-desktop.sh 0.1.0" >&2
  exit 1
fi

if ! command -v cargo-tauri >/dev/null 2>&1; then
  echo "::error::cargo-tauri is not installed." >&2
  echo "  cargo install tauri-cli --version '^2' --locked" >&2
  exit 1
fi

# The window's title bar, the About box, and the installer all read this, so it
# has to be the release's version rather than whatever is committed. Tauri
# reads it from tauri.conf.json, which is checked in — so it is overridden on
# the command line rather than by rewriting a tracked file mid-build.
echo "==> building Hickory Docs $VERSION for $TARGET"

# `hick` reports its version from this at compile time; the app embeds the same
# engine, so the two agree about what they are.
export HICKORY_VERSION="$VERSION"

# What the BUNDLE records as its version, which is not always what we call the
# build. Windows rejects `0.1.0-unstable.f957b9e4` outright — an MSI's optional
# pre-release identifier must be numeric-only and no greater than 65535 — and
# the whole desktop release fails at the bundling step, after every binary has
# already been built. So the installer records the numeric core and the FILE
# name still carries the full identifier, which is the half a person reads.
#
# Only Windows: macOS and Linux take the full string, and an app that knows
# exactly which unstable build it is is worth keeping where it is allowed.
# `hick --version` reports HICKORY_VERSION on every platform regardless.
case "$TARGET" in
  *windows*) BUNDLE_VERSION="${VERSION%%-*}" ;;
  *)         BUNDLE_VERSION="$VERSION" ;;
esac
if [ "$BUNDLE_VERSION" != "$VERSION" ]; then
  echo "==> msi records $BUNDLE_VERSION; the file is named $VERSION"
fi

# The UI is built by tauri.conf.json's beforeBuildCommand (`npm run build` in
# apps/web), so node_modules has to exist first. Doing it here keeps the
# failure legible: "no node_modules" beats a build script exiting 127.
if [ ! -d apps/web/node_modules ]; then
  echo "==> installing frontend dependencies"
  npm --prefix apps/web ci
fi

# Run from the crate directory: tauri.conf.json's `beforeBuildCommand` and
# `frontendDist` are relative to the config file, but cargo-tauri resolves them
# against the working directory. From the repo root they would point one level
# above the repo.
cross_args=()
if [ "$TARGET" != "$HOST_TARGET" ]; then
  cross_args=(--target "$TARGET")
fi
(
  cd apps/desktop/src-tauri
  # `CI=true` is what makes the macOS `.dmg` buildable anywhere.
  #
  # Tauri passes `--skip-jenkins` to its bundled `bundle_dmg.sh` only when `CI`
  # is `true`; without it the script drives **Finder over AppleScript** to
  # prettify the disk image's window, which fails on any machine where that is
  # not permitted or where there is no usable GUI session — measured 2026-08-18
  # on a MacBook Pro, where a local run died with
  # `Finder got an error: AppleEvent timed out. (-1712)` after the app bundle
  # had already been built successfully.
  #
  # Setting it unconditionally also means a local build and a release build
  # produce the *same* disk image, which is what
  # `continuous-delivery-downloadable.md` asks for: the artifact CI verifies is
  # the artifact a user gets. All that is given up is the icon layout inside the
  # `.dmg` window, which CI was already giving up.
  #
  # macOS runners ship bash 3.2, where expanding an empty array under `set -u`
  # is an "unbound variable" error; the ${arr[@]+...} form is the portable
  # spelling of "expand only if non-empty".
  CI=true cargo tauri build \
    --config "{\"version\": \"$BUNDLE_VERSION\"}" \
    ${cross_args[@]+"${cross_args[@]}"} \
    ${TAURI_BUNDLES:+--bundles "$TAURI_BUNDLES"}
)

# Tauri writes bundles under the crate's own target directory, in a
# per-format subdirectory. Collect whatever this platform produced rather than
# naming formats here: the matrix decides which platform runs, and the
# platform decides what a bundle looks like.
# A cross build lands under target/<triple>/release instead.
if [ "$TARGET" != "$HOST_TARGET" ]; then
  BUNDLE_DIR="apps/desktop/src-tauri/target/$TARGET/release/bundle"
else
  BUNDLE_DIR="apps/desktop/src-tauri/target/release/bundle"
fi
if [ ! -d "$BUNDLE_DIR" ]; then
  echo "::error::cargo tauri build reported success but $BUNDLE_DIR does not exist." >&2
  echo "If the bundle output moved, update this script and the release workflow" >&2
  echo "together — the workflow uploads exactly what lands in dist/desktop." >&2
  exit 1
fi

OUT=dist/desktop
rm -rf "$OUT"
mkdir -p "$OUT"

# Tauri names bundles from productName, which is "Hickory Docs" — with a
# space, and capitalised differently per format. A published asset name is a
# contract (scripts/install.sh already lives by that for the CLI), and a space
# in a URL is a paper cut every downloader pays. So every bundle is renamed to
# one shape:  hickory-docs-<version>-<target>.<ext>
found=0
while IFS= read -r bundle; do
  case "$bundle" in
    *.deb|*.rpm|*.AppImage|*.dmg|*.msi|*.exe) ;;
    *.app.tar.gz) ;;
    *) continue ;;
  esac
  case "$bundle" in
    *.app.tar.gz) ext="app.tar.gz" ;;
    *) ext="${bundle##*.}" ;;
  esac
  cp "$bundle" "$OUT/hickory-docs-$VERSION-$TARGET.$ext"
  found=$((found + 1))
done < <(find "$BUNDLE_DIR" -type f)

if [ "$found" -eq 0 ]; then
  echo "::error::no installable bundle was produced under $BUNDLE_DIR." >&2
  echo "Contents:" >&2
  find "$BUNDLE_DIR" -maxdepth 2 >&2
  exit 1
fi

# Checksums travel with the artifact, as they do for the CLI.
for file in "$OUT"/*; do
  case "$file" in
    *.sha256) continue ;;
  esac
  name="$(basename "$file")"
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$OUT" && sha256sum "$name" > "$name.sha256")
  elif command -v shasum >/dev/null 2>&1; then
    (cd "$OUT" && shasum -a 256 "$name" > "$name.sha256")
  else
    echo "::error::no sha256 tool found (tried sha256sum, shasum)." >&2
    exit 1
  fi
done

echo "==> $found bundle(s) in $OUT"
ls -l "$OUT"

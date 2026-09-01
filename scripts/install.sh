#!/bin/sh
# One-line installer for the `hick` CLI.
#
#   curl -fsSL https://raw.githubusercontent.com/LoumTechnologies/hickory-docs/master/scripts/install.sh | sh
#
# Downloads the release archive matching this machine, checks its SHA-256
# against the checksum published beside it, and puts `hick` — and the editor
# language server `hick-lsp` — on your PATH. No Rust toolchain, no clone, no
# build.
#
# Knobs (all optional, all environment variables so the piped-into-sh form
# still works — `curl … | HICKORY_CHANNEL=unstable sh`):
#   HICKORY_CHANNEL      stable (default) | unstable
#   HICKORY_VERSION      exact release tag, e.g. v0.1.0 (overrides CHANNEL)
#   HICKORY_INSTALL_DIR  where to put the binary (default: ~/.local/bin)
#   HICKORY_GITHUB_TOKEN a GitHub token, for as long as the repo is private
#
# Deliberately POSIX sh, not bash: this is the first thing a stranger runs,
# and it must not depend on anything their machine might not have.
set -eu

REPO="LoumTechnologies/hickory-docs"
API="https://api.github.com/repos/$REPO"
CHANNEL="${HICKORY_CHANNEL:-stable}"
INSTALL_DIR="${HICKORY_INSTALL_DIR:-$HOME/.local/bin}"

die() {
  echo "" >&2
  echo "hick install failed: $1" >&2
  shift
  for line in "$@"; do echo "  $line" >&2; done
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 || die \
    "this installer needs \`$1\`, which is not on your PATH." \
    "Install it with your package manager and re-run, or download the" \
    "archive by hand from https://github.com/$REPO/releases"
}

need curl
need tar
need uname

# --- which artifact does this machine want? ---------------------------------

os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
  Linux)  triple_os="unknown-linux-gnu" ;;
  Darwin) triple_os="apple-darwin" ;;
  MINGW*|MSYS*|CYGWIN*|Windows_NT)
    die "Windows is shipped as a .zip, not through this script." \
      "Download hick-<version>-x86_64-pc-windows-msvc.zip from" \
      "https://github.com/$REPO/releases, unzip it, and put hick.exe on" \
      "your PATH. Note that running documents needs a POSIX \`sh\` — Git Bash" \
      "or WSL both provide one."
    ;;
  *)
    die "unsupported operating system: $os" \
      "Supported: Linux, macOS. Windows has a .zip on the releases page." \
      "Everything else can build from source: cargo build --release -p hickory-cli"
    ;;
esac

case "$arch" in
  x86_64|amd64)  triple_arch="x86_64" ;;
  aarch64|arm64) triple_arch="aarch64" ;;
  *)
    die "unsupported CPU architecture: $arch" \
      "Prebuilt binaries exist for x86_64 and aarch64 only." \
      "On anything else, build from source: cargo build --release -p hickory-cli"
    ;;
esac

TARGET="$triple_arch-$triple_os"

# --- which release? ---------------------------------------------------------

fetch() {
  # $1 = url, $2 = Accept header, $3 = output file ("-" for stdout)
  if [ -n "${HICKORY_GITHUB_TOKEN:-}" ]; then
    curl -fsSL -H "Accept: $2" -H "Authorization: Bearer $HICKORY_GITHUB_TOKEN" \
      -o "$3" "$1"
  else
    curl -fsSL -H "Accept: $2" -o "$3" "$1"
  fi
}

if [ -n "${HICKORY_VERSION:-}" ]; then
  release_url="$API/releases/tags/$HICKORY_VERSION"
  what="$HICKORY_VERSION"
elif [ "$CHANNEL" = "unstable" ]; then
  # The unstable channel is a single rolling tag, replaced by every green
  # build of master — not a growing pile of prereleases.
  release_url="$API/releases/tags/unstable"
  what="the unstable channel"
elif [ "$CHANNEL" = "stable" ]; then
  release_url="$API/releases/latest"
  what="the latest stable release"
else
  die "unknown channel: $CHANNEL" "Valid values are 'stable' and 'unstable'."
fi

tmp="$(mktemp -d 2>/dev/null || mktemp -d -t hick)"
trap 'rm -rf "$tmp"' EXIT INT TERM

echo "Looking up $what for ${TARGET}…"
if ! fetch "$release_url" "application/vnd.github+json" "$tmp/release.json"; then
  if [ "$CHANNEL" = "stable" ] && [ -z "${HICKORY_VERSION:-}" ]; then
    # Pre-1.0 there may be no stable release at all: /releases/latest
    # excludes prereleases, so the default install would die on a repo whose
    # only release is the rolling `unstable` tag. Fall back to it loudly
    # rather than telling a stranger the product cannot be installed.
    echo "No stable release found; falling back to the unstable channel."
    echo "(Every unstable build passed the same CI; pin one later with HICKORY_VERSION.)"
    release_url="$API/releases/tags/unstable"
    what="the unstable channel"
    fetch "$release_url" "application/vnd.github+json" "$tmp/release.json" || die \
      "could not read $what from GitHub." \
      "URL: $release_url" \
      "If the repository is private, set HICKORY_GITHUB_TOKEN to a token with" \
      "read access and re-run. If you are offline or behind a proxy, that would" \
      "show up here too."
  else
    die \
      "could not read $what from GitHub." \
      "URL: $release_url" \
      "If the repository is private, set HICKORY_GITHUB_TOKEN to a token with" \
      "read access and re-run. If you are offline or behind a proxy, that would" \
      "show up here too."
  fi
fi

# No jq: this script is the first thing a stranger runs, before they have
# installed anything at all.
#
# Strip newlines, then split on `{`, so each asset object lands on exactly one
# line with its `url` and `name` fields together. (GitHub emits an asset's
# fields in the order url, id, node_id, name, label, uploader{…} — the nested
# uploader object is what ends the line, after the two fields wanted here.)
tr -d '\n' < "$tmp/release.json" | tr '{' '\n' > "$tmp/assets"

field() {
  # $1 = one asset line, $2 = key. Anchored to the start of the field so
  # asking for `url` cannot answer with `browser_download_url`.
  printf '%s' "$1" | tr ',' '\n' | sed -n "s/^ *\"$2\": *\"\\([^\"]*\\)\".*/\\1/p" | head -1
}

asset_name="hick-*-$TARGET.tar.gz"
line="$(grep '"name": *"hick-' "$tmp/assets" | grep -- "-$TARGET\.tar\.gz\"" | head -1 || true)"

[ -n "$line" ] || die \
  "$what has no artifact for this machine ($TARGET)." \
  "Expected an asset named like $asset_name." \
  "See what was published: https://github.com/$REPO/releases"

file="$(field "$line" name)"
accept="application/octet-stream"

if [ -n "${HICKORY_GITHUB_TOKEN:-}" ]; then
  # The API asset URL is the only one that accepts a token; the browser URL
  # redirects to a signed CDN link that rejects the Authorization header.
  url="$(field "$line" url)"
  sum_line="$(grep "\"name\": *\"$file\.sha256\"" "$tmp/assets" | head -1 || true)"
  sum_url="$(field "$sum_line" url)"
else
  # Composed rather than read back: `browser_download_url` sits after the
  # nested uploader object and so is not on this line. It is a stable,
  # documented URL shape, and composing it keeps the parsing to one field.
  tag="$(sed -n 's/^ *"tag_name": *"\([^"]*\)".*/\1/p' "$tmp/release.json" | head -1)"
  [ -n "$tag" ] || die "the GitHub response for $what had no tag_name." \
    "That usually means a rate limit or an error body came back instead." \
    "Retry, or set HICKORY_GITHUB_TOKEN to raise the rate limit."
  url="https://github.com/$REPO/releases/download/$tag/$file"
  sum_url="$url.sha256"
fi

[ -n "$url" ] || die "found the asset entry for $TARGET but no download URL in it." \
  "This usually means the GitHub API response was truncated or rate-limited." \
  "Try again, or set HICKORY_GITHUB_TOKEN to raise the rate limit."

# --- download and verify ----------------------------------------------------

echo "Downloading ${file}…"
fetch "$url" "$accept" "$tmp/$file" || die \
  "download failed: $url" \
  "Check your network, then retry. The archive can also be fetched by hand" \
  "from https://github.com/$REPO/releases"

if [ -n "$sum_url" ] && fetch "$sum_url" "$accept" "$tmp/$file.sha256" 2>/dev/null; then
  expected="$(cut -d' ' -f1 < "$tmp/$file.sha256")"
  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$tmp/$file" | cut -d' ' -f1)"
  elif command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "$tmp/$file" | cut -d' ' -f1)"
  else
    actual=""
    echo "warning: no sha256sum/shasum on this machine — skipping checksum check" >&2
  fi
  if [ -n "$actual" ] && [ "$actual" != "$expected" ]; then
    die "checksum mismatch for $file — refusing to install it." \
      "expected: $expected" \
      "actual:   $actual" \
      "A truncated download is the usual cause; re-run to retry. If it keeps" \
      "failing, report it at https://github.com/$REPO/issues"
  fi
else
  echo "warning: no published checksum for $file — installing unverified" >&2
fi

# --- install ----------------------------------------------------------------

tar -xzf "$tmp/$file" -C "$tmp"
extracted="$tmp/$(basename "$file" .tar.gz)/hick"
[ -f "$extracted" ] || die \
  "the archive did not contain a \`hick\` binary where one was expected." \
  "Looked for: $extracted" \
  "This is a packaging bug — please report it at https://github.com/$REPO/issues"

mkdir -p "$INSTALL_DIR" || die \
  "could not create $INSTALL_DIR." \
  "Set HICKORY_INSTALL_DIR to somewhere writable, e.g." \
  "  curl … | HICKORY_INSTALL_DIR=\$HOME/bin sh"

install -m 755 "$extracted" "$INSTALL_DIR/hick" 2>/dev/null ||
  { cp "$extracted" "$INSTALL_DIR/hick" && chmod 755 "$INSTALL_DIR/hick"; } || die \
  "could not write $INSTALL_DIR/hick." \
  "Is it writable? Set HICKORY_INSTALL_DIR to somewhere that is."

echo ""
echo "Installed: $INSTALL_DIR/hick"
"$INSTALL_DIR/hick" --version || true

# The editor language server travels in the same archive. It is not required
# to run documents, so a release that predates it installs `hick` and says
# what is missing rather than failing.
lsp="$tmp/$(basename "$file" .tar.gz)/hick-lsp"
if [ -f "$lsp" ]; then
  install -m 755 "$lsp" "$INSTALL_DIR/hick-lsp" 2>/dev/null ||
    { cp "$lsp" "$INSTALL_DIR/hick-lsp" && chmod 755 "$INSTALL_DIR/hick-lsp"; } || die \
    "could not write $INSTALL_DIR/hick-lsp." \
    "Is it writable? Set HICKORY_INSTALL_DIR to somewhere that is."
  echo "Installed: $INSTALL_DIR/hick-lsp (editor language server)"
else
  echo "note: this release carries no hick-lsp; editor diagnostics inside" >&2
  echo "      .hick files need a newer one (HICKORY_CHANNEL=unstable)." >&2
fi

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo ""
    echo "$INSTALL_DIR is not on your PATH. Add this to your shell profile:"
    echo ""
    echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
    ;;
esac

# Cells run confined by default; on Linux that needs bubblewrap. Saying so
# now beats the first `hick test` refusing on a machine that was just told
# the install succeeded.
if [ "$(uname -s)" = "Linux" ] && ! command -v bwrap >/dev/null 2>&1; then
  echo "" >&2
  echo "note: bubblewrap is not installed, and hick runs document cells" >&2
  echo "      sandboxed by default. Install it before running a document:" >&2
  echo "          sudo apt install bubblewrap    # or dnf/pacman/zypper" >&2
  echo "      (or set HICKORY_EXECUTOR=local to run unconfined at your own risk)" >&2
fi

EXAMPLES_DIR="${INSTALL_DIR%/bin}/share/hick/examples"
mkdir -p "$(dirname "$EXAMPLES_DIR")" 2>/dev/null || true
rm -rf "$EXAMPLES_DIR"
if cp -R "$tmp/$(basename "$file" .tar.gz)/examples" "$EXAMPLES_DIR" 2>/dev/null; then
  echo ""
  echo "Try it:  hick test $EXAMPLES_DIR/text-tools-tour.hick"
  echo "(the whole set — $EXAMPLES_DIR — also needs python3, polars, and the"
  echo " duckdb CLI; text-tools-tour needs only a POSIX shell.)"
fi

echo ""
echo "In a repo:  hick init"
echo "(sets up the drift gate, registers the MCP server for your coding agent,"
echo " and wires hick-lsp into the editors it finds)"
echo ""
echo "Docs: https://github.com/$REPO#reference"

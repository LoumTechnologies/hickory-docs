#!/usr/bin/env bash
# Does a download work on a machine that has never had a toolchain?
#
# Protects docs/guarantees/release/a-download-runs-without-a-rust-toolchain.md
#
# Runs on a guest reverted to its golden snapshot, so "no Rust" is a property of
# the machine rather than of a step that uninstalled it. The existing
# `clean-machine` CI job removes rustup from a GitHub runner and is a good
# proxy; it still runs on an image that shipped with Node, Python, a package
# manager and a C toolchain. This is the real thing.
#
# Linux and macOS guests. Windows is vmtest/scripts/download.ps1 — it promises
# less, and says so.
set -uo pipefail

# The helpers live INSIDE this directory because vmkit pushes the script's
# directory into the guest and nothing above it. A sibling `../lib` resolves on
# the host and is absent in the guest — where, worse, a script that cannot
# source its helpers still finishes without ever printing a RESULT line, and the
# run is reported as a PASS. So this is checked before anything else, loudly.
lib="$(dirname "$0")/lib/assert.sh"
if [ ! -f "$lib" ]; then
    echo "RESULT=FAIL the assertion helpers were not pushed with this script ($lib)"
    exit 1
fi
# shellcheck source=vmtest/scripts/lib/assert.sh
. "$lib"

# `prlctl exec` hands over HOME=/ on Unix guests (vmkit CAPABILITIES), and
# anything reading a state directory would write into / and fail confusingly.
export HOME="${HOME:-/}"
[ "$HOME" = "/" ] && export HOME="/tmp/hickory-home"
mkdir -p "$HOME"

archive="${HICKORY_ARCHIVE:-}"
if [ -z "$archive" ] || [ ! -f "$archive" ]; then
    vmkit_skip "no archive staged (HICKORY_ARCHIVE=${archive:-unset}) — build one and stage it at vmkit.conf's VMKIT_ARTIFACT_* path"
fi

work="$(mktemp -d)"
cd "$work" || vmkit_skip "no writable scratch directory"

# --- the machine itself ------------------------------------------------------
# Asserted, not assumed. A guest that has quietly acquired a toolchain still
# passes every later phase while proving nothing, and the whole point of a
# pristine VM is the absence.
assert_not pristine-no-cargo command -v cargo
assert_not pristine-no-rustc command -v rustc
assert_not pristine-no-node command -v node

# --- unpack ------------------------------------------------------------------
assert unpack tar -xzf "$archive"
root="$(find . -maxdepth 1 -type d -name 'hick-*' | head -1)"
if [ -z "$root" ]; then
    echo "PHASE=unpack-layout ok=false"
    vmkit_result "the archive did not unpack to a hick-* directory"
fi
echo "PHASE=unpack-layout ok=true"
hick="$root/hick"

# Everything the guarantee says the archive carries, not just the binary: the
# examples are what `hick test` verifies against, and shipping them without
# their committed outputs would ship something unverifiable.
assert carries-binary test -x "$hick"
assert carries-lsp test -x "$root/hick-lsp"
assert carries-licence test -f "$root/LICENSE"
assert carries-readme test -f "$root/README.md"
assert carries-examples test -d "$root/examples"
assert carries-example-outputs test -f "$root/examples/text-tools-tour.md"

# --- the binary runs ---------------------------------------------------------
assert version "$hick" --version
assert help "$hick" --help

# The version the binary reports must equal the version it was published as —
# the asset-name contract in continuous-delivery-downloadable.md. Checked only
# when CI said what to expect; by hand there is nothing to compare against.
if [ -n "${HICKORY_EXPECT_VERSION:-}" ]; then
    reported="$("$hick" --version 2>/dev/null | tr -d '\r')"
    case "$reported" in
        *"$HICKORY_EXPECT_VERSION"*) echo "PHASE=version-matches-asset ok=true" ;;
        *)
            echo "PHASE=version-matches-asset ok=false"
            echo "   expected '$HICKORY_EXPECT_VERSION', binary said '$reported'"
            VMKIT_FAILS=$((VMKIT_FAILS + 1))
            ;;
    esac
fi

# `hick-lsp` speaks LSP over stdio and has no --version; closed stdin is the one
# input that makes it exit rather than wait, so an exit code proves it loads.
assert lsp-loads sh -c "printf '' | '$PWD/$root/hick-lsp'"

# --- it does the job ---------------------------------------------------------
# ORDER MATTERS, and getting it wrong makes the archive look broken when it is
# not. `hick weave` writes the document's `.md` beside it, so weaving inside the
# unpacked archive REPLACES the committed output with a never-run rendering —
# and `hick test`, which verifies re-derived output against that committed copy,
# then reports drift against a file this script clobbered a moment earlier. The
# first run of this test "found" exactly that and it was not a product bug.
#
# So: verify first, against the bytes as shipped.
doc=examples/text-tools-tour.hick
woven=examples/text-tools-tour.md
if ( cd "$root" && "./hick" test "$doc" ) > exec.log 2>&1; then
    echo "PHASE=execute-a-document ok=true"
else
    echo "PHASE=execute-a-document ok=false"
    sed -n '1,20p' exec.log
    # Say WHAT drifted, not just that something did. `hick test` reports the
    # outcome and exits; on a machine nobody can log into, "committed file
    # differs" without the difference is a report you cannot act on — and the
    # likeliest cause is a tool that behaves differently here, which the diff
    # names immediately.
    cp "$root/$woven" committed.md 2>/dev/null
    ( cd "$root" && "./hick" run "$doc" ) >/dev/null 2>&1
    echo "-- committed vs produced on this guest:"
    diff -u committed.md "$root/$woven" 2>/dev/null | sed -n '1,40p' || true
    VMKIT_FAILS=$((VMKIT_FAILS + 1))
fi

# …and only then weave, into a scratch directory of its own so it cannot write
# over anything the archive shipped. Weave renders from cached transcripts and
# executes nothing, which makes it the half that works with no shell at all.
mkdir -p "$work/woven"
assert weave "$hick" weave "$root/$doc" --out "$work/woven"
assert weave-produced-a-file test -s "$work/woven/text-tools-tour.md"

# Which sandbox actually confined it. Recorded rather than asserted: the
# executor's own guarantee covers confinement, and what matters here is that a
# downloaded binary finds one on a machine nobody prepared.
echo "-- executor on this guest: $("$hick" --version 2>/dev/null) / $(uname -s) $(uname -m)"

vmkit_result "download on a pristine $(uname -s) guest"

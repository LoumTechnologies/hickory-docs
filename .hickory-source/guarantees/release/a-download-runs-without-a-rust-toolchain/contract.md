<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Download Runs Without A Rust Toolchain

For: Maintainers publishing installers and people checking what a download contains.

Given a machine with no Rust toolchain, no C compiler, and no checkout of this
repository, when a person downloads the release archive matching their
platform and unpacks it, then `hick --version` and `hick test examples/`
both work — using only the archive's own contents plus whatever the example
documents themselves invoke.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/a-download-runs-without-a-rust-toolchain.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

<a id="output-6990843536db"></a>
## `scripts/dist.sh`

Related contracts: [A Download Runs Without A Rust Toolchain](contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="scripts/dist.sh" executable="true">
#!/usr/bin/env bash
# Build one downloadable `hick` artifact.
#
# Usage: scripts/dist.sh <target-triple> [version]
#
# Called identically by `just dist` and by the release workflows, so what a
# maintainer can reproduce locally is byte-for-byte what CI ships (same as
# scripts/check-codegen.sh serving both the hook and CI).
#
# Output, under dist/:
#   hick-<version>-<target>.tar.gz   (unix)  or  .zip (windows)
#   hick-<version>-<target>.<ext>.sha256
#
# The archive holds both binaries — `hick` and the editor language server
# `hick-lsp` — plus README, LICENCE, and examples/, so a stranger who
# downloads it can run `hick test examples/` immediately and point an editor
# at `hick-lsp` without a Rust toolchain. That is the acceptance test for this
# whole path (see
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

# `hick --version` should report the version of the artifact somebody
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

echo "==> building hick $VERSION for $TARGET"
# Both binaries, one build: `hick` is the tool, `hick-lsp` is what an editor
# runs to get real diagnostics inside a .md document. Shipping only the
# first would mean docs/users/editor-setup.md asking a downloader for a Rust
# toolchain, which is the one thing this artifact exists to avoid.
cargo build --release --locked -p hickory-cli -p hick-lsp --target "$TARGET"

SUFFIX=
EXT=tar.gz
case "$TARGET" in
  *-windows-*)
    SUFFIX=.exe
    EXT=zip
    ;;
esac

BIN="hick$SUFFIX"
LSP_BIN="hick-lsp$SUFFIX"

for built in "$BIN" "$LSP_BIN"; do
  if [ ! -f "target/$TARGET/release/$built" ]; then
    echo "::error::cargo reported success but target/$TARGET/release/$built does not exist." >&2
    echo "The [[bin]] names in crates/hickory-cli/Cargo.toml and" >&2
    echo "crates/hick-lsp/Cargo.toml are what these paths are derived from;" >&2
    echo "if one was renamed, update scripts/dist.sh and scripts/install.sh" >&2
    echo "together — the installer resolves the same names." >&2
    exit 1
  fi
done

NAME="hick-$VERSION-$TARGET"
STAGE="dist/$NAME"
rm -rf "$STAGE"
mkdir -p "$STAGE"

cp "target/$TARGET/release/$BIN" "$STAGE/$BIN"
cp "target/$TARGET/release/$LSP_BIN" "$STAGE/$LSP_BIN"
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
</storycode:entry>

<a id="output-c2be09976208"></a>
## `vmtest/scripts/download.ps1`

Does a download work on a Windows machine that has never had a toolchain?
Protects docs/guarantees/release/a-download-runs-without-a-rust-toolchain.md
This promises LESS than the Unix script, on purpose. A document's cells run
through `sh`, and Windows has no guarantee of one -- Git Bash and WSL both
provide it, and a runner having either proves nothing about a user's machine.
So execution is asserted where a shell is genuinely present and reported as a
skip naming the reason where it is not, rather than being dropped from the
matrix where nobody would notice its absence.
`prlctl exec` runs this as NT AUTHORITY\SYSTEM (vmkit CAPABILITIES), which is
the identity an MSI's deferred custom actions get -- good parity for an
installer, and it means no mapped drive letters and no user profile.
THIS FILE IS ASCII ONLY, and that is load-bearing rather than a style choice.
Windows PowerShell 5.1 reads a UTF-8 file with no BOM as CP1252, so an em dash
(E2 80 94) arrives as `a`, `EUR`, `"` -- and that last one is U+201D, a smart
quote PowerShell honours as a string delimiter. One em dash inside a
double-quoted message therefore ends the string early and the rest of the file
parses as code: the first run of this died on
`Missing closing '}' in statement block` pointing at a line whose braces were
balanced. `scripts/check-guest-scripts.sh` keeps it ASCII.
The helpers live INSIDE this directory because vmkit pushes the script's
directory into the guest and nothing above it. A sibling `..\lib` resolves on
the host and is absent in the guest -- where a script that cannot load them
finishes without printing a RESULT line and the run is reported as a PASS.

Related contracts: [A Download Runs Without A Rust Toolchain](contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="vmtest/scripts/download.ps1">
# Does a download work on a Windows machine that has never had a toolchain?
#
# Protects docs/guarantees/release/a-download-runs-without-a-rust-toolchain.md
#
# This promises LESS than the Unix script, on purpose. A document's cells run
# through `sh`, and Windows has no guarantee of one -- Git Bash and WSL both
# provide it, and a runner having either proves nothing about a user's machine.
# So execution is asserted where a shell is genuinely present and reported as a
# skip naming the reason where it is not, rather than being dropped from the
# matrix where nobody would notice its absence.
#
# `prlctl exec` runs this as NT AUTHORITY\SYSTEM (vmkit CAPABILITIES), which is
# the identity an MSI's deferred custom actions get -- good parity for an
# installer, and it means no mapped drive letters and no user profile.
#
# THIS FILE IS ASCII ONLY, and that is load-bearing rather than a style choice.
# Windows PowerShell 5.1 reads a UTF-8 file with no BOM as CP1252, so an em dash
# (E2 80 94) arrives as `a`, `EUR`, `"` -- and that last one is U+201D, a smart
# quote PowerShell honours as a string delimiter. One em dash inside a
# double-quoted message therefore ends the string early and the rest of the file
# parses as code: the first run of this died on
# `Missing closing '}' in statement block` pointing at a line whose braces were
# balanced. `scripts/check-guest-scripts.sh` keeps it ASCII.

# The helpers live INSIDE this directory because vmkit pushes the script's
# directory into the guest and nothing above it. A sibling `..\lib` resolves on
# the host and is absent in the guest -- where a script that cannot load them
# finishes without printing a RESULT line and the run is reported as a PASS.
$lib = Join-Path $PSScriptRoot 'lib\assert.ps1'
if (-not (Test-Path $lib)) {
    Write-Output "RESULT=FAIL the assertion helpers were not pushed with this script ($lib)"
    exit 1
}
. $lib

# Run a native program, capturing its exit code AND its output.
#
# `Invoke-Guarded` from the shared library deliberately answers only
# "completed or timed out": it runs the block in a background JOB, which is a
# separate process, so `$LASTEXITCODE` and any variable the block sets do not
# come back. Everything below needs the exit code, so this uses
# `Start-Process -Wait` with explicit log files -- which CAPABILITIES.md
# recommends anyway, because `| Out-Null` on a native command can hang through
# pipe inheritance.
#
# `-Wait` is required, not incidental. `Start-Process -PassThru` WITHOUT it
# hands back a process object whose `ExitCode` is never populated: every phase
# that checked one failed with a blank code while the command had plainly
# worked, and `weave` reported failure with its output file sitting on disk.
#
# There is deliberately no timeout here. vmkit already owns that -- the flavor's
# `VMKIT_FLAVOR_DOWNLOAD_TIMEOUT`, plus `VMKIT_KILL_WINDOWS` to reap a straggler
# afterwards -- and a second, weaker timeout inside the guest bought nothing
# except the broken exit code above.
function Invoke-Native {
    param(
        [string]   $Exe,
        [string[]] $Arguments = @(),
        [string]   $WorkDir = $PWD.Path,
        [string]   $StdIn
    )
    $log = Join-Path $env:TEMP ("hickory-" + [guid]::NewGuid().ToString('N') + ".log")
    $err = "$log.err"
    $start = @{
        FilePath               = $Exe
        WorkingDirectory       = $WorkDir
        RedirectStandardOutput = $log
        RedirectStandardError  = $err
        NoNewWindow            = $true
        PassThru               = $true
        Wait                   = $true
    }
    if ($Arguments.Count -gt 0) { $start['ArgumentList'] = $Arguments }
    if ($StdIn) { $start['RedirectStandardInput'] = $StdIn }

    # A missing file makes Start-Process THROW and return nothing, so $proc is
    # null and every call on it fails with a stack trace instead of a verdict.
    # The published Windows zip not carrying hick-lsp.exe is exactly that case,
    # and a harness whose own error hides the finding is worse than no harness.
    $proc = $null
    try {
        $proc = Start-Process @start -ErrorAction Stop
    } catch {
        [Console]::Error.WriteLine("   Start-Process failed for $Exe : $($_.Exception.Message)")
        return @{ Completed = $false; ExitCode = -1; Out = ''; Err = '' }
    }
    if (-not $proc) {
        return @{ Completed = $false; ExitCode = -1; Out = ''; Err = '' }
    }

    $stdout = if (Test-Path $log) { (Get-Content $log -Raw) } else { '' }
    $stderr = if (Test-Path $err) { (Get-Content $err -Raw) } else { '' }
    return @{
        Completed = $true
        ExitCode  = $proc.ExitCode
        Out       = ("$stdout").Trim()
        Err       = ("$stderr").Trim()
    }
}

# Diagnostics go to STDERR, and that is not a stylistic preference. Anything a
# PowerShell function writes to the success stream becomes part of its return
# value, so a `Write-Output` here made this return an array of (message, bool)
# -- `Phase` then failed to bind its `[bool]` parameter, threw, and the phase
# vanished from the report entirely. A helper that reports a failure by deleting
# it is the worst possible failure mode for a test harness.
function Ran-Ok($result) {
    if (-not $result.Completed) { return $false }
    if ($result.ExitCode -ne 0) {
        [Console]::Error.WriteLine("   exit code $($result.ExitCode): $($result.Err)")
        return $false
    }
    return $true
}

$archive = $env:HICKORY_ARCHIVE
if (-not $archive -or -not (Test-Path $archive)) {
    Vmkit-Skip "no archive staged (HICKORY_ARCHIVE=$archive) -- build one and stage it at vmkit.conf's VMKIT_ARTIFACT_WINDOWS path"
}

$work = Join-Path $env:TEMP ("hickory-download-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $work -Force | Out-Null
Set-Location $work

# --- the machine itself ------------------------------------------------------
# The absence is the point, so it is asserted rather than assumed. A guest that
# has quietly acquired a toolchain passes every later phase while proving
# nothing.
Phase 'pristine-no-cargo' (-not [bool](Get-Command cargo -ErrorAction SilentlyContinue))
Phase 'pristine-no-rustc' (-not [bool](Get-Command rustc -ErrorAction SilentlyContinue))
Phase 'pristine-no-node'  (-not [bool](Get-Command node  -ErrorAction SilentlyContinue))

# --- unpack ------------------------------------------------------------------
$unpacked = $false
try {
    Expand-Archive -LiteralPath $archive -DestinationPath $work -Force
    $unpacked = $true
} catch {
    Write-Output "   Expand-Archive failed: $_"
}
Phase 'unpack' $unpacked

$root = Get-ChildItem -Path $work -Directory -Filter 'hick-*' | Select-Object -First 1
if (-not $root) {
    Phase 'unpack-layout' $false
    Vmkit-Result 'the archive did not unpack to a hick-* directory'
}
Phase 'unpack-layout' $true

$hick = Join-Path $root.FullName 'hick.exe'
$lsp  = Join-Path $root.FullName 'hick-lsp.exe'

# Everything the guarantee says the archive carries, not just the binary.
Phase 'carries-binary'          (Test-Path $hick)
Phase 'carries-lsp'             (Test-Path $lsp)
Phase 'carries-licence'         (Test-Path (Join-Path $root.FullName 'LICENSE'))
Phase 'carries-readme'          (Test-Path (Join-Path $root.FullName 'README.md'))
Phase 'carries-examples'        (Test-Path (Join-Path $root.FullName 'examples'))
Phase 'carries-example-outputs' (Test-Path (Join-Path $root.FullName 'examples\text-tools-tour.md'))

# --- the binary runs ---------------------------------------------------------
$version = Invoke-Native -Exe $hick -Arguments @('--version')
Phase 'version' (Ran-Ok $version)
Write-Output "   reported: $($version.Out)"

Phase 'help' (Ran-Ok (Invoke-Native -Exe $hick -Arguments @('--help')))

# The binary must agree with the version it was published under -- the
# asset-name contract in continuous-delivery-downloadable.md.
if ($env:HICKORY_EXPECT_VERSION) {
    $matches = $version.Out -like "*$($env:HICKORY_EXPECT_VERSION)*"
    Phase 'version-matches-asset' $matches
    if (-not $matches) {
        Write-Output "   expected '$($env:HICKORY_EXPECT_VERSION)', binary said '$($version.Out)'"
    }
}

# `hick-lsp` speaks LSP over stdio and has no --version. Closed stdin is the one
# input that makes it exit rather than wait, so an exit code proves it loads --
# hence an empty file redirected in, which reaches it as EOF.
$empty = Join-Path $work 'empty.txt'
New-Item -ItemType File -Path $empty -Force | Out-Null
Phase 'lsp-loads' (Ran-Ok (Invoke-Native -Exe $lsp -StdIn $empty))

# --- it does the job ---------------------------------------------------------
# ORDER MATTERS. `hick weave` writes the document's `.md` beside it, so weaving
# inside the unpacked archive replaces the committed output with a never-run
# rendering -- and `hick test` then reports drift against a file this script
# clobbered. Verify first, against the bytes as shipped; weave afterwards, into
# a directory of its own.
$doc = Join-Path $root.FullName 'examples\text-tools-tour.md'

# The SHIPPED EXAMPLES cannot run here, and the reason is not the one this
# script used to give.
#
# It claimed a document's cells shell out to `sh`, which Windows does not ship.
# That is wrong: `LocalExecutor::shell()` returns `cmd.exe /C` on Windows, so
# cells run perfectly well. What cannot run is these particular documents --
# `text-tools-tour` is `printf`, `sort -t, -k2 -n` and `awk`, none of which cmd
# has. The limitation belongs to the examples, not to the product.
#
# So: assert it where a POSIX shell happens to exist, and otherwise skip naming
# the real reason. Windows cell execution proper is covered by the `sandbox`
# flavor, which uses a cmd-syntax document.
if (Get-Command sh -ErrorAction SilentlyContinue) {
    $run = Invoke-Native -Exe $hick -Arguments @('test', 'examples\text-tools-tour.md') `
        -WorkDir $root.FullName
    Phase 'execute-a-document' (Ran-Ok $run)
    if (-not (Ran-Ok $run)) {
        Write-Output "   $($run.Out)"
        Write-Output "   $($run.Err)"
    }
} else {
    Phase-Skip 'execute-a-document' "the shipped examples are POSIX shell (printf/sort/awk) and this machine has none; cells themselves run through cmd.exe -- see the sandbox flavor for Windows cell execution"
}

# Weave executes nothing, so it is the half that works with no shell at all --
# on a Windows machine without one it is the whole product surface that still
# has to work.
$woven = Join-Path $work 'woven'
New-Item -ItemType Directory -Path $woven -Force | Out-Null
Phase 'weave' (Ran-Ok (Invoke-Native -Exe $hick -Arguments @('weave', $doc, '--out', $woven)))
Phase 'weave-produced-a-file' (Test-Path (Join-Path $woven 'text-tools-tour.md'))

Vmkit-Result 'download on a pristine Windows guest'
</storycode:entry>

<a id="output-b1e567c61721"></a>
## `vmtest/scripts/download.sh`

Related contracts: [A Download Runs Without A Rust Toolchain](contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="vmtest/scripts/download.sh" executable="true">
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
doc=examples/text-tools-tour.md
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
</storycode:entry>

## Implementation review

### Implementation

- [crates/hickory-cli/src/init.rs](../../../mechanisms/crates/hickory-cli/src/init.rs.md#output-a329ec22d0b8)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-executor-canopy/src/executor.rs](../../../mechanisms/crates/hickory-executor-canopy/src/executor.rs.md#output-47988291742d)
- [scripts/dist.sh](contract.md#output-6990843536db)
- [scripts/install.sh](../../../mechanisms/scripts/install.sh.md#output-e076e43cb817)
- [vmtest/scripts/download.ps1](contract.md#output-c2be09976208)
- [vmtest/scripts/download.sh](contract.md#output-b1e567c61721)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>

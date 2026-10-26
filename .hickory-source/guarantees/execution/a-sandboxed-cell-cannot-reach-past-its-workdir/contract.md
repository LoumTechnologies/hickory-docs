<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Sandboxed Cell Cannot Reach Past Its Workdir

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given `HICKORY_EXECUTOR=sandbox`, when a document's cells run, then each cell
may write only its own container workdir and a private `/tmp`; it sees an
empty home directory rather than yours; it cannot read another container's
workdir; and it has no network unless the document declared one with
`<hick:allow>`.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)
- [Requirements 3](requirements/03.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-sandboxed-cell-cannot-reach-past-its-workdir.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />
- [Requirement detail](requirements/03.md)
<storycode:include file="requirements/03.md" />

## Implementation

## Test the behavior: `per_shell`

This is the maintained test fragment from `crates/hickory-executor-sandbox/tests/confinement.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-c8a04313c63c-per_shell"></a>
### `per_shell`

Does the sandbox actually confine anything?
Protects docs/guarantees/execution/a-sandboxed-cell-cannot-reach-past-its-workdir.md
These run the real sandbox against the real filesystem, because the only
interesting question is whether a hostile cell is stopped — and that is
not observable from the argv. Each is skipped, loudly, on a machine with
no sandbox: a silent skip would let this file rot into a decoration.

<storycode:copy id="f-c8a04313c63c-per_shell">
//! Does the sandbox actually confine anything?
//!
//! Protects docs/guarantees/execution/a-sandboxed-cell-cannot-reach-past-its-workdir.md
//!
//! These run the real sandbox against the real filesystem, because the only
//! interesting question is whether a hostile cell is stopped — and that is
//! not observable from the argv. Each is skipped, loudly, on a machine with
//! no sandbox: a silent skip would let this file rot into a decoration.

use hickory_executor::Executor;
use hickory_executor_sandbox::{Sandbox, SandboxedExecutor};

/// The same intent, in the shell a cell actually gets on this platform.
///
/// Cells run through `sh -c` on Unix and `cmd.exe /C` on Windows, so a test
/// written in POSIX syntax is not testing the sandbox on Windows — it is
/// testing whether Git for Windows happens to be on PATH. And when it is, the
/// test still fails, for a reason worth knowing: msys2 binaries need a section
/// object in the GLOBAL `\BaseNamedObjects` namespace, and an AppContainer is
/// given a private one, so `cat`, `sleep` and the rest die with
/// STATUS_ACCESS_DENIED before doing anything at all. Measured in CI,
/// 2026-08-19 — see the guarantee. Git-Bash tooling simply cannot run confined
/// on Windows.
fn per_shell(unix: &str, windows: &str) -> String {
    if cfg!(windows) {
        windows.to_string()
    } else {
        unix.to_string()
    }
}

</storycode:copy>

<a id="output-48ea736fda79"></a>
## `vmtest/scripts/sandbox.ps1`

Does the Windows sandbox actually confine a cell?
Protects docs/guarantees/execution/a-sandboxed-cell-cannot-reach-past-its-workdir.md
AppContainer is the newest of the three confinements and, until this ran, the
only one never executed anywhere: CI is Linux and macOS, and the release job
smoke-tests the Windows binary with `--version`. `appcontainer.rs` compiles and
`Sandbox::detect` returns it on every Windows machine, so the guarantee's
Windows sentences were read off the source rather than observed.
This drives the SHIPPED binary rather than `cargo test`, because a pristine
guest has no toolchain and because what ships is what a user runs. The crate's
own confinement.rs covers the same properties for whoever has a Windows dev
machine.
ASCII ONLY -- see download.ps1 for why (PowerShell 5.1 + CP1252 + smart quotes).

Related contracts: [A Sandboxed Cell Cannot Reach Past Its Workdir](contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="vmtest/scripts/sandbox.ps1">
# Does the Windows sandbox actually confine a cell?
#
# Protects docs/guarantees/execution/a-sandboxed-cell-cannot-reach-past-its-workdir.md
#
# AppContainer is the newest of the three confinements and, until this ran, the
# only one never executed anywhere: CI is Linux and macOS, and the release job
# smoke-tests the Windows binary with `--version`. `appcontainer.rs` compiles and
# `Sandbox::detect` returns it on every Windows machine, so the guarantee's
# Windows sentences were read off the source rather than observed.
#
# This drives the SHIPPED binary rather than `cargo test`, because a pristine
# guest has no toolchain and because what ships is what a user runs. The crate's
# own confinement.rs covers the same properties for whoever has a Windows dev
# machine.
#
# ASCII ONLY -- see download.ps1 for why (PowerShell 5.1 + CP1252 + smart quotes).

$lib = Join-Path $PSScriptRoot 'lib\assert.ps1'
if (-not (Test-Path $lib)) {
    Write-Output "RESULT=FAIL the assertion helpers were not pushed with this script ($lib)"
    exit 1
}
. $lib

function Invoke-Native {
    param([string] $Exe, [string[]] $Arguments = @(), [string] $WorkDir = $PWD.Path,
          [hashtable] $Env = @{})
    $log = Join-Path $env:TEMP ("hickory-" + [guid]::NewGuid().ToString('N') + ".log")
    $err = "$log.err"
    foreach ($k in $Env.Keys) { Set-Item -Path "env:$k" -Value $Env[$k] }
    $start = @{
        FilePath = $Exe; WorkingDirectory = $WorkDir
        RedirectStandardOutput = $log; RedirectStandardError = $err
        NoNewWindow = $true; PassThru = $true; Wait = $true
    }
    if ($Arguments.Count -gt 0) { $start['ArgumentList'] = $Arguments }
    $proc = $null
    try { $proc = Start-Process @start -ErrorAction Stop } catch {
        [Console]::Error.WriteLine("   Start-Process failed for $Exe : $($_.Exception.Message)")
        return @{ Completed = $false; ExitCode = -1; Out = ''; Err = '' }
    }
    $o = if (Test-Path $log) { (Get-Content $log -Raw) } else { '' }
    $e = if (Test-Path $err) { (Get-Content $err -Raw) } else { '' }
    return @{ Completed = $true; ExitCode = $proc.ExitCode; Out = ("$o").Trim(); Err = ("$e").Trim() }
}

$archive = $env:HICKORY_ARCHIVE
if (-not $archive -or -not (Test-Path $archive)) {
    Vmkit-Skip "no archive staged (HICKORY_ARCHIVE=$archive)"
}

$work = Join-Path $env:TEMP ("hickory-sandbox-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $work -Force | Out-Null
Set-Location $work
Expand-Archive -LiteralPath $archive -DestinationPath $work -Force
$root = Get-ChildItem -Path $work -Directory -Filter 'hick-*' | Select-Object -First 1
$hick = Join-Path $root.FullName 'hick.exe'
Phase 'binary-present' (Test-Path $hick)

# The escape target, as an absolute path the HOST can check afterwards. Asserting
# on the file's absence rather than on the cell's exit code is the point: an
# installer that "succeeded" while writing nowhere is the failure mode a sandbox
# has to rule out.
$escape = Join-Path $env:USERPROFILE 'hickory-should-not-exist.txt'
Remove-Item -LiteralPath $escape -ErrorAction SilentlyContinue

# Cells run through `cmd.exe /C` on Windows (LocalExecutor::shell), NOT `sh` --
# so this document is cmd syntax. The shipped POSIX examples cannot run here,
# which is why they are not what this flavor uses.
$doc = @"
<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="confined.md">
# Confinement

<hick:container name="c" image="windows" />

<hick:exec container="c">
echo confined > note.txt & type note.txt
</hick:exec>

<hick:exec container="c">
echo escaped > "$escape" & echo WROTE
</hick:exec>
</hick:doc>
"@
$docPath = Join-Path $work 'confined.md'
# UTF-8 WITHOUT a BOM. PowerShell 5.1's `-Encoding UTF8` writes one, and a
# leading BOM used to make hick parse a wrapped document as a bare one -- the
# cells never ran and the run still exited 0. That is fixed in the parser, but
# this test exists to exercise the SANDBOX; writing the file the way an editor
# that does not add a BOM would keeps the two failures apart, so a red run here
# means confinement is broken rather than encoding.
[System.IO.File]::WriteAllText(
    $docPath, $doc, (New-Object System.Text.UTF8Encoding($false)))

$run = Invoke-Native -Exe $hick -Arguments @('run', $docPath) -WorkDir $work `
    -Env @{ HICKORY_EXECUTOR = 'sandbox' }
Write-Output "   run exit=$($run.ExitCode)"
if ($run.Out) { Write-Output "   out: $($run.Out)" }
if ($run.Err) { Write-Output "   err: $($run.Err)" }

$woven = Join-Path $work 'confined.md'
$wovenText = if (Test-Path $woven) { Get-Content $woven -Raw } else { '' }
$cellRan = [bool]($wovenText -match 'confined')
# Phrases the executor uses when it declines to run rather than run
# unconfined. Deliberately NOT a bare 'sandbox' match: this tool says the word
# in its own configuration errors, so 'unknown HICKORY_EXECUTOR value
# "sandbox"' -- a run that never started -- once satisfied this assertion.
$refused = [bool]($run.Err -match 'cannot confine container|no sandbox available')

# 0. The run reached a cell at all. Everything below is about what a cell was
#    prevented from doing, and a run that never got that far did not do any of
#    it either: a document that failed at startup writes nothing anywhere, so
#    "nothing escaped" is true of it and means nothing. This assertion exists
#    so that a stale or broken archive reads as a broken archive rather than as
#    a working sandbox.
Phase 'the-archive-can-run-a-confined-document' ($cellRan -or $refused)
if (-not $cellRan) {
    # What the run actually produced. Without this the failure is "the woven
    # document did not contain the word", which says nothing about whether the
    # cell was denied, produced no output, or never ran.
    Write-Output "   --- $woven ---"
    if (Test-Path $woven) {
        foreach ($l in ($wovenText -split "`r?`n")) { Write-Output "   | $l" }
    } else {
        Write-Output "   (no woven file at all)"
    }
    $note = Join-Path $work 'note.txt'
    Write-Output "   note.txt exists: $(Test-Path $note)"
    Get-ChildItem -Path $work -Recurse -File -ErrorAction SilentlyContinue |
        Select-Object -First 20 |
        ForEach-Object { Write-Output "   file: $($_.FullName) ($($_.Length) bytes)" }
}

# 1. A confined cell RUNS. If AppContainer cannot start a process at all, every
#    other assertion here would pass vacuously.
Phase 'a-confined-cell-runs' $cellRan

# 2. It cannot write outside its workdir. Checked on the HOST filesystem, not on
#    the cell's exit code -- an installer that "succeeded" while writing nowhere
#    is exactly the failure a sandbox has to rule out.
if ($cellRan) {
    Phase 'a-cell-cannot-write-outside-its-workdir' (-not (Test-Path $escape))
    if (Test-Path $escape) {
        Write-Output "   the sandbox let a cell create $escape"
        Remove-Item -LiteralPath $escape -ErrorAction SilentlyContinue
    }
} else {
    Phase-Skip 'a-cell-cannot-write-outside-its-workdir' 'no cell ran, so nothing could have escaped'
}

# 3. The executor did not quietly fall back to running unconfined. A refusal is
#    an acceptable outcome for this guarantee; pretending is not.
Phase 'did-not-silently-run-unconfined' ($cellRan -or $refused)

Vmkit-Result 'Windows AppContainer confinement'
</storycode:entry>

## Implementation review

### Implementation

- [crates/hickory-executor-sandbox/src/appcontainer.rs](../every-desktop-platform-can-confine-a-cell/contract.md#output-bfbb07ecda29)
- [vmtest/scripts/sandbox.ps1](contract.md#output-48ea736fda79)

### Tests and fixtures

- [crates/hickory-cli/tests/lsp_languages.rs](../../../mechanisms/crates/hickory-cli/tests/lsp_languages.rs.md#output-7c4650d33fc8)
- [crates/hickory-executor-sandbox/tests/confinement.rs](../../../mechanisms/crates/hickory-executor-sandbox/tests/confinement.rs.md#output-c8a04313c63c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>

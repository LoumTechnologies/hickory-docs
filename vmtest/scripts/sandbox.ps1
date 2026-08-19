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
$docPath = Join-Path $work 'confined.hick'
Set-Content -LiteralPath $docPath -Value $doc -Encoding UTF8

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

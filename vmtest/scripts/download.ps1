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
$doc = Join-Path $root.FullName 'examples\text-tools-tour.hick'

# Execution needs `sh`. Where the machine has one this is a real assertion;
# where it does not it is a skip that names why.
if (Get-Command sh -ErrorAction SilentlyContinue) {
    $run = Invoke-Native -Exe $hick -Arguments @('test', 'examples\text-tools-tour.hick') `
        -WorkDir $root.FullName
    Phase 'execute-a-document' (Ran-Ok $run)
    if (-not (Ran-Ok $run)) {
        Write-Output "   $($run.Out)"
        Write-Output "   $($run.Err)"
    }
} else {
    Phase-Skip 'execute-a-document' "no 'sh' on this machine; a document's cells shell out to one and Windows does not ship it (docs/users/install.md)"
}

# Weave executes nothing, so it is the half that works with no shell at all --
# on a Windows machine without one it is the whole product surface that still
# has to work.
$woven = Join-Path $work 'woven'
New-Item -ItemType Directory -Path $woven -Force | Out-Null
Phase 'weave' (Ran-Ok (Invoke-Native -Exe $hick -Arguments @('weave', $doc, '--out', $woven)))
Phase 'weave-produced-a-file' (Test-Path (Join-Path $woven 'text-tools-tour.md'))

Vmkit-Result 'download on a pristine Windows guest'

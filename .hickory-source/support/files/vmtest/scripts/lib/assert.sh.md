<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# vmtest/scripts/lib/assert.sh

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `8405edd9867f3deb0882d57c14414d8deaffc6cf6d166fd82524e6af8e2b7cff`.

<book:entry path="vmtest/scripts/lib/assert.sh">
# shellcheck shell=bash
# vmkit guest-side assertion helpers (bash; linux + macos guests).
#
# Copy this file into your repo next to your flavor scripts (guests can't
# reach the host's vmkit install) and source it:
#     . "$(dirname "$0")/lib/assert.sh"
#
# Protocol: print greppable "PHASE=<name> ok=true|false|SKIP" lines as you go,
# end with vmkit_result. The host-side series runner surfaces these verbatim.
#
# Helper predicates are usually invoked indirectly via assert/assert_not;
# silence the resulting spurious shellcheck SC2317 "unreachable" info in your
# script with: # shellcheck disable=SC2317

VMKIT_FAILS=0

# assert <phase> <cmd...>: PHASE ok=true if the command succeeds.
assert() {
    local phase="$1"; shift
    if "$@" >/dev/null 2>&1; then echo "PHASE=$phase ok=true"
    else echo "PHASE=$phase ok=false"; VMKIT_FAILS=$((VMKIT_FAILS + 1)); fi
}

# assert_not <phase> <cmd...>: PHASE ok=true if the command FAILS (absence checks).
assert_not() {
    local phase="$1"; shift
    if "$@" >/dev/null 2>&1; then echo "PHASE=$phase ok=false"; VMKIT_FAILS=$((VMKIT_FAILS + 1))
    else echo "PHASE=$phase ok=true"; fi
}

# assert_or_skip <phase> <reason> <cmd...>: never counts as a failure — ok=true
# when the predicate holds, else ok=SKIP with the reason. Reserve for RUNTIME
# states a headless guest genuinely cannot establish (see docs/CAPABILITIES.md:
# e.g. macOS System-keychain trust settings and `launchctl bootstrap system`
# both fail under headless prlctl exec) — and make sure some OTHER test covers
# the skipped behavior for real.
assert_or_skip() {
    local phase="$1" reason="$2"; shift 2
    if "$@" >/dev/null 2>&1; then echo "PHASE=$phase ok=true"
    else echo "PHASE=$phase ok=SKIP reason=\"$reason\""; fi
}

# vmkit_result <description>: prints RESULT=PASS/FAIL and exits accordingly.
vmkit_result() {
    if [ "$VMKIT_FAILS" -eq 0 ]; then
        echo "RESULT=PASS ${1:-}"
        exit 0
    fi
    echo "RESULT=FAIL ${1:-} assertions failed=$VMKIT_FAILS"
    exit 1
}

# vmkit_skip <reason>: whole-script SKIP (e.g. a prerequisite artifact is
# unavailable). SKIP exits 0 — never a false failure — but always says why.
vmkit_skip() {
    echo "RESULT=SKIP ${1:-}"
    exit 0
}
</book:entry>

</book:doc>

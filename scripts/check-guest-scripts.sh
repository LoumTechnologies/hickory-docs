#!/usr/bin/env bash
# The guest test scripts parse on the guest, not here.
#
# Both halves of this were real failures on the first run of `vmkit test`:
#
#  * A `.ps1` containing one em dash died with
#    `Missing closing '}' in statement block` on a line whose braces were
#    balanced. Windows PowerShell 5.1 reads a UTF-8 file with no BOM as CP1252,
#    so `—` arrives as three characters ending in U+201D — a smart quote that
#    PowerShell treats as a string delimiter, which ends the enclosing string
#    early and turns the rest of the file into code.
#  * A `.sh` with a syntax error would fail the same way, minutes into a VM run
#    rather than in a second here.
#
# Cheap enough to run on every commit, and the alternative is finding out after
# a guest has booted.
set -euo pipefail
cd "$(dirname "$0")/.."

status=0

for script in vmtest/scripts/*.sh vmtest/scripts/lib/*.sh; do
    [ -e "$script" ] || continue
    if ! bash -n "$script"; then
        echo "::error file=$script::shell syntax error"
        status=1
    fi
done

for script in vmtest/scripts/*.ps1 vmtest/scripts/lib/*.ps1; do
    [ -e "$script" ] || continue
    if LC_ALL=C grep -n '[^ -~	]' "$script" >/dev/null 2>&1; then
        echo "::error file=$script::non-ASCII character in a PowerShell script."
        echo "  Windows PowerShell 5.1 reads UTF-8 without a BOM as CP1252, and an"
        echo "  em dash decodes to a smart quote it treats as a string delimiter —"
        echo "  which ends a string early and makes the whole file fail to parse."
        echo "  Offending lines:"
        LC_ALL=C grep -n '[^ -~	]' "$script" | sed 's/^/    /'
        status=1
    fi
done

if [ "$status" -eq 0 ]; then
    echo "guest scripts: shell syntax ok, PowerShell ASCII-only"
fi
exit "$status"

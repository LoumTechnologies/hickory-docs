<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain scripts/checks.sh

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-510b44f6e3a4"></a>
## `scripts/checks.sh`

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="scripts/checks.sh" executable="true">
#!/usr/bin/env bash
# Bootstrap only: the supervisor is dependency-free Rust, outside the workspace
# so a broken application cannot prevent checks from starting.
set -euo pipefail
cd "$(dirname "$0")/.."
runner=$(mktemp "${TMPDIR:-/tmp}/hickory-checks.XXXXXX")
trap 'rm -f "$runner"' EXIT
rustc --edition=2021 --crate-name hickory_checks scripts/checks/main.rs -o "$runner"
if [ "${1:-}" = --test ]; then
  tests=$(mktemp "${TMPDIR:-/tmp}/hickory-checks-tests.XXXXXX")
  trap 'rm -f "$runner" "$tests"' EXIT
  HICKORY_CHECKS_SOURCE_ROOT="$PWD" rustc --edition=2021 --crate-name hickory_checks --test scripts/checks/main.rs -o "$tests"
  HICKORY_CHECKS_TEST_RUNNER="$runner" "$tests" --test-threads=1
else
  "$runner" "$@"
fi
</storycode:entry>

</storycode:doc>

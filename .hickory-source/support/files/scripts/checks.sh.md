<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# scripts/checks.sh

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `5464a3db1e3b0387b3b4a378180fe683d8a909953ee4028b0e1fe9fdd1643509`.

<book:entry path="scripts/checks.sh" executable="true">
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
</book:entry>

</book:doc>

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

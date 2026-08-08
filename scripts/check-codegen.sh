#!/usr/bin/env bash
# CI + pre-commit drift check: regenerate, then fail if anything changed that
# isn't already recorded. Compares against the index, not HEAD, so a freshly
# `git add`ed-but-correct generated file doesn't false-fail.
# Usage: scripts/check-codegen.sh   (via `just check-codegen`)
set -euo pipefail
cd "$(dirname "$0")/.."

GENERATED_PATHS=(apps/server/openapi.json apps/web/src/api/generated)

./scripts/codegen.sh

unstaged=$(git diff --name-only -- "${GENERATED_PATHS[@]}")
untracked=$(git ls-files --others --exclude-standard -- "${GENERATED_PATHS[@]}")

if [ -n "$unstaged" ] || [ -n "$untracked" ]; then
  echo
  echo "Generated API client is out of date."
  echo "Run 'just codegen', then commit apps/server/openapi.json and apps/web/src/api/generated/."
  echo
  echo "Changed:"
  printf '%s\n' "$unstaged" "$untracked" | sed '/^$/d'
  exit 1
fi

echo "Generated API client is up to date."

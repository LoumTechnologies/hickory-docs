#!/usr/bin/env bash
# The one shared "which checks does this diff need" mapping. Called
# identically by .githooks/pre-commit and .github/workflows/ci.yml so the two
# can never disagree about what a change requires.
#
# Usage: scripts/affected-checks.sh <base-ref>
#   <base-ref> is diffed against the working tree (not a triple-dot range),
#   so a hook run against a merge-base picks up staged+unstaged changes, and
#   a CI run on a clean checkout (working tree == HEAD) sees exactly the
#   committed diff.
#
# Prints one check name per line. No output at all means nothing in the
# mapping matched (e.g. a docs-only change) — the caller should run nothing,
# not fall back to running everything.
set -euo pipefail
cd "$(dirname "$0")/.."

BASE_REF="${1:?usage: affected-checks.sh <base-ref>}"

changed=$(git diff --name-only "$BASE_REF" -- . 2>/dev/null || true)
if [ -z "$changed" ]; then
  exit 0
fi

checks=()

matches() {
  local pattern="$1"
  while IFS= read -r f; do
    # shellcheck disable=SC2053
    if [[ "$f" == $pattern ]]; then
      return 0
    fi
  done <<<"$changed"
  return 1
}

# Declarative path-glob -> checks table. A change touching multiple rows
# runs the union of their checks.
if matches "crates/*" || matches "Cargo.toml" || matches "Cargo.lock"; then
  checks+=(rust)
fi
if matches "apps/web/*"; then
  checks+=(web)
fi
# Not a docs-lint row: this repo's "rust" job also verifies docs/examples
# drift (`hickory-cli check docs/`, `check examples/`) — a docs-only change
# genuinely needs that, unlike a typical markdown-lint-only repo.
if matches "docs/*" || matches "examples/*"; then
  checks+=(rust)
fi
# The desktop crate is deliberately outside the cargo workspace, so the
# `rust` check never touches it — and the release path (dist scripts,
# workflow definitions) used to have NO row at all, which is exactly how two
# release-breaking bugs reached master with green hooks and green CI.
if matches "apps/desktop/*" || matches "scripts/dist*.sh" || matches ".github/workflows/*"; then
  checks+=(desktop)
fi
# Expensive on purpose: the dev environment itself is only re-verified when
# something that can actually break it changes, not on every commit. See
# docs/developers/developer-environment.md.
#
# `dev` is the row that runs the dev scripts themselves. Before it existed,
# `scripts/dev*.sh` mapped to rust+web — checks that build the things the dev
# environment starts without ever executing the scripts that start them, so a
# broken seed or a stale-fixture bug had nothing standing in its way.
if matches "docker-compose.yml" || matches "scripts/dev*.sh" || matches "justfile"; then
  checks+=(rust web dev)
fi

if [ "${#checks[@]}" -gt 0 ]; then
  printf '%s\n' "${checks[@]}" | sort -u
fi

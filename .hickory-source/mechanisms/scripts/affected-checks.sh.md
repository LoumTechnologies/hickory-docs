<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain scripts/affected-checks.sh

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5adf98eb496e"></a>
## `scripts/affected-checks.sh`

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="scripts/affected-checks.sh" executable="true">
#!/usr/bin/env bash
# The one shared "which checks does this diff need" mapping. Called
# identically by just checks and .github/workflows/ci.yml so the two
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

changed=$(mktemp)
trap 'rm -f "$changed"' EXIT
git diff --name-only -z "$BASE_REF" -- . > "$changed"
git ls-files --others --exclude-standard -z >> "$changed"
if [ ! -s "$changed" ]; then
  exit 0
fi

checks=()

matches() {
  local pattern="$1"
  while IFS= read -r -d '' f; do
    # shellcheck disable=SC2053
    if [[ "$f" == $pattern ]]; then
      return 0
    fi
  done < "$changed"
  return 1
}

# Declarative path-glob -> checks table. A change touching multiple rows
# runs the union of their checks.
if matches "crates/*" || matches "Cargo.toml" || matches "Cargo.lock" || matches "rust-toolchain.toml" || matches ".cargo/*" || matches "scripts/check-codegen.sh"; then
  checks+=(rust)
fi
if matches "apps/web/*"; then
  checks+=(web)
fi
if matches "scripts/test-python-agent-ui.sh" || matches "apps/web/e2e/python-agent.spec.ts" || matches "crates/hickory-cli/examples/python_agent_ui_fixture.rs" || matches "apps/web/src/components/Agent*" || matches "apps/web/src/components/ChatDock*" || matches "apps/web/src/components/AcpControls*" || matches "apps/web/src/views/AgentSettings*" || matches "apps/web/src/api/acp.ts" || matches "apps/agent/*" || matches "scripts/check-python-agent.sh" || matches "crates/hickory-cli/src/python_agent*" || matches "crates/hickory-cli/src/agent_command.rs" || matches "crates/hickory-cli/tests/python_agent.rs" || matches "crates/hickory-cli/tests/serve_python_agent.rs" || matches "crates/hickory-cli/src/serve/python_agent.rs" || matches "crates/hickory-cli/src/serve/agent*" || matches "crates/hickory-cli/src/serve/acp/*" || matches "crates/hickory-cli/src/serve/mod.rs" || matches "crates/hickory-cli/src/mcp*" || matches "crates/hickory-agent/*" || matches "crates/hickory-executor-sandbox/*"; then
  checks+=(python-agent)
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
if matches "apps/desktop/*" || matches "scripts/dist*.sh" || matches "scripts/install.sh" || matches ".github/workflows/*"; then
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
if matches "docker-compose.yml" || matches "scripts/dev*.sh" || matches "scripts/check-dev-seed.sh" || matches "justfile"; then
  checks+=(rust web dev)
fi

# Changes to the gate or its shared selector must exercise every branch.
if matches ".githooks/*" || matches "scripts/affected-checks.sh" || matches "scripts/checks*"; then
  checks+=(rust web desktop dev python-agent)
fi
if matches "scripts/check-file-length.sh" || matches "scripts/file-length-baseline.txt"; then
  checks+=(rust web)
fi

if [ "${#checks[@]}" -gt 0 ]; then
  printf '%s\n' "${checks[@]}" | sort -u
fi
</storycode:entry>

</storycode:doc>

#!/usr/bin/env bash
# Idempotent seed data: a scratch project of real documents for `just dev` to
# open. Usage: scripts/dev-seed.sh   (via `just dev-seed`)
#
# There are no accounts to seed — no server, no database, nobody to log in as
# (docs/specs/freeform/local-only.md). What a developer needs on a fresh
# checkout is a folder the app can open with something interesting in it, so
# that is what this makes: a two-stage chain, because a single document cannot
# show the lineage browser doing its job.
#
# Safe to run twice: existing files are left exactly as they are, so a
# document you edited while developing survives the next `just dev`.
set -euo pipefail
cd "$(dirname "$0")/.."

PROJECT_DIR=.dev/project
mkdir -p "$PROJECT_DIR"

write_if_absent() {
  local path="$1"
  if [ -e "$path" ]; then
    echo "  kept    $path"
    return
  fi
  mkdir -p "$(dirname "$path")"
  cat >"$path"
  echo "  created $path"
}

echo "Seeding $PROJECT_DIR"

write_if_absent "$PROJECT_DIR/decisions.hick" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="decisions.md">
# Scratch project — decisions

The upstream end of a two-stage chain. Editing a fragment here changes what
the next document weaves, which is the whole point of the lineage browser.

<hick:copy id="d-format" class="decision">
Durations are seconds, as floats. No units in field names, no strings, no
ISO 8601 — one representation, chosen once.
</hick:copy>

<hick:copy id="d-empty" class="decision">
An empty input is not an error. It produces an empty result, because the
first run of a fresh install must behave like every later run.
</hick:copy>
</hick:doc>
EOF

write_if_absent "$PROJECT_DIR/stats.hick" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="stats.md">
# Scratch project — implementation

<hick:upstream file="decisions.hick" />

## Decisions in force

<hick:paste select=".decision" />

## The code

<hick:copy id="c-mean" class="stats-py">
def mean(values):
    """Seconds in, seconds out — see the format decision upstream."""
    if not values:
        return 0.0
    return sum(values) / len(values)
</hick:copy>

<hick:copy id="c-main" class="stats-py">
if __name__ == "__main__":
    print(mean([0.5, 1.5, 2.5]))
</hick:copy>

<hick:file path="stats.py" language="python">
<hick:paste select=".stats-py" />
</hick:file>

## It runs

<hick:container name="py" image="python:3.12" />
<hick:volume name="src" input="." />

<hick:exec container="py" mount="src:project">
cd project && python3 stats.py
<hick:expect match="exact">1.5
</hick:expect>
</hick:exec>
</hick:doc>
EOF

# Weave once, so a fresh seed is CONSISTENT rather than drifted. Without this
# the first thing a developer might try — `hick test .dev/project` — reports a
# failure that is really just "nothing has run yet", which is a bad first
# impression of the drift gate.
HICK=""
for candidate in target/release/hick target/debug/hick; do
  [ -x "$candidate" ] && HICK="$candidate" && break
done
[ -n "$HICK" ] || HICK="$(command -v hick || true)"
if [ -n "$HICK" ]; then
  echo
  echo "Weaving the seed with $HICK…"
  "$HICK" run "$PROJECT_DIR" >/dev/null 2>&1 || echo "  (weave skipped — the app will do it on open)"
fi

echo
echo "Seeded. \`just dev\` opens this folder; \`just dev-clean\` removes it."

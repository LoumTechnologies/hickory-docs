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

write_if_absent "$PROJECT_DIR/debugging.hick" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="debugging.md">
# Scratch project — the debugger

A document you can *stop inside*. Everything below is meant to be tried in the
app rather than read: the editor is a real Python editor here, the gutter takes
breakpoints, and the last cell records values from inside a function without
anybody stepping at all.

## The program

Editing this block is editing `orders.py`. Hover a name for its type, ctrl-space
for completions, and misspell something to watch the squiggle appear — the
language server is the real one, running against the file this weaves.

<hick:file path="orders.py" language="python">
ORDERS = [
    ("widget", 2, 9.99),
    ("gizmo", 1, 24.50),
    ("doohickey", 12, 1.25),
]

BULK_THRESHOLD = 10
BULK_DISCOUNT = 0.15


def line_total(name, quantity, unit_price):
    """What one line of the order is worth, after any bulk discount."""
    subtotal = quantity * unit_price
    if quantity >= BULK_THRESHOLD:
        subtotal = subtotal * (1 - BULK_DISCOUNT)
    return round(subtotal, 2)


def order_total(lines):
    return round(sum(line_total(*line) for line in lines), 2)


if __name__ == "__main__":
    print(f"total {order_total(ORDERS):.2f}")
</hick:file>

## Stopping inside it

Click the gutter beside `subtotal = quantity * unit_price` to leave a red dot,
then start the debugger. The paused line gets an arrow, values appear at the end
of every line that mentions one, and hovering a name shows what it holds *now*
on top of what the language server says it is.

Step in, over and out; run to the cursor; evaluate anything you like in the
frame you are stopped in. Backwards works too, as far as the adapter allows —
on Python that means moving the instruction pointer, which **re-runs** the line
rather than rewinding it, and the control says so.

Nothing you do in there touches this folder: the debugger runs a copy.

## Recording what it saw, with nobody watching

The cell below runs the program the ordinary way. The captures are breakpoints
the *document* owns — each hit evaluates the same expressions the interactive
pane would, and the values are woven in underneath, where an expectation block
can pin them.

<hick:container name="py" />
<hick:volume name="src" input="." />

<hick:exec container="py" mount="src:project">
cd project && python3 orders.py
  <hick:capture at="orders.py:15" of="name, quantity, subtotal" />
  <hick:capture at="orders.py:17" of="name, subtotal" condition="quantity >= BULK_THRESHOLD" />
</hick:exec>

A breakpoint stops *before* its line runs, so both captures sit one line below
the assignment they are about: the first records the plain subtotal for every
line of the order, and the second records the discounted one — but only for the
bulk line, which is the claim worth making about this program and the one no
amount of checking stdout can reach.
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

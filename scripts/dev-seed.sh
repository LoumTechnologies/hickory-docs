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
# Safe to run twice, and re-run by every `just dev` — which is the point. A
# seed that only ever wrote missing files meant that changing a fixture in this
# repository left every existing checkout showing the old one, with nothing on
# screen to say so and `just dev-clean` as the undocumented cure. So each file
# is written with a record of what was seeded, and a re-seed can tell the two
# cases apart:
#
#   * you have not touched it  -> it is replaced with the current fixture,
#   * you edited it            -> your version is kept, and the difference is
#                                 reported by name rather than left to surprise
#                                 you later.
set -euo pipefail
cd "$(dirname "$0")/.."

# Both are overridable so this script can be exercised against a throwaway
# directory (scripts/check-dev-seed.sh) without touching a developer's real
# scratch project. Nothing but that check sets them.
SEED_ROOT="${HICKORY_SEED_ROOT:-.dev}"
PROJECT_DIR="$SEED_ROOT/project"
MANIFEST="$SEED_ROOT/seed-manifest"
# The weave at the end builds a binary, which the check does not need and
# should not pay for. 1 for a real seed, 0 for the check.
SEED_WEAVE="${HICKORY_SEED_WEAVE:-1}"
mkdir -p "$PROJECT_DIR" "$SEED_ROOT"
touch "$MANIFEST"

# A newline-separated list rather than an array: macOS still ships bash 3.2,
# where an empty array expanded under `set -u` is an error.
STALE=""

# sha256 of stdin, on every platform a developer here might have: coreutils,
# macOS, and git-bash all ship one of these three.
hash_stdin() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum | cut -d' ' -f1
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 | cut -d' ' -f1
  else
    openssl dgst -sha256 | awk '{print $NF}'
  fi
}

recorded_hash() {
  awk -v p="$1" '$2 == p { print $1 }' "$MANIFEST" | tail -1
}

record_hash() {
  local path="$1" hash="$2" tmp
  tmp=$(mktemp)
  awk -v p="$path" '$2 != p' "$MANIFEST" >"$tmp"
  echo "$hash $path" >>"$tmp"
  mv "$tmp" "$MANIFEST"
}

# Write a seeded file, or explain why it was left alone. Reads the content on
# stdin; the manifest is what makes the "unmodified" case safe to overwrite.
seed_file() {
  local path="$1" content seeded current recorded
  content=$(cat)
  seeded=$(printf '%s\n' "$content" | hash_stdin)

  if [ ! -e "$path" ]; then
    mkdir -p "$(dirname "$path")"
    printf '%s\n' "$content" >"$path"
    record_hash "$path" "$seeded"
    echo "  created $path"
    return
  fi

  current=$(hash_stdin <"$path")
  if [ "$current" = "$seeded" ]; then
    record_hash "$path" "$seeded"
    echo "  current $path"
    return
  fi

  recorded=$(recorded_hash "$path")
  if [ -n "$recorded" ] && [ "$current" = "$recorded" ]; then
    printf '%s\n' "$content" >"$path"
    record_hash "$path" "$seeded"
    echo "  updated $path (the fixture changed; you had not edited it)"
    return
  fi

  STALE="$STALE$path\n"
  echo "  kept    $path (your edits — the fixture in git differs)"
}

echo "Seeding $PROJECT_DIR"

seed_file "$PROJECT_DIR/decisions.md" <<'EOF'
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

seed_file "$PROJECT_DIR/stats.md" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="stats.md">
# Scratch project — implementation

<hick:upstream file="decisions.md" />

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

seed_file "$PROJECT_DIR/debugging.md" <<'EOF'
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

seed_file "$PROJECT_DIR/workspace.md" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="workspace.md">
# Workspace

A layout, declared. Each block below names a region of this folder and the
files that belong to it; the app arranges one pane per region and opens a file
into the region whose globs claim it.

Structure only — no widths, no tabs, nothing about what is open. Those are
session state, and this file would change every time somebody dragged a
splitter. See docs/specs/freeform/shell-layouts.md.

<hick:copy id="documents" class="layout-region">
*.md
</hick:copy>

<hick:copy id="generated" class="layout-region">
*.md
*.py
</hick:copy>
</hick:doc>
EOF

# The card fixture: every piece of editor chrome at once, so the rail, the
# inline chips, and the two fold kinds can be looked at together rather than
# hunted for across four documents. Its job is the gutter guarantee — see
# docs/guarantees/authoring/the-gutters-never-skip-a-number.md — which is only
# checkable by eye, on a document that exercises all of it.
seed_file "$PROJECT_DIR/cards.md" <<'EOF'
# Every card, in one document

A fixture, not a tutorial. It exists so the editor's card UI can be looked at
all at once: every rail icon, every inline chip, every banner, and both fold
kinds, in a document short enough to scroll in one pass.

**No wrapper.** This document starts at its first heading: no XML
declaration, no root element, no closing tag. It weaves `cards.md` because
that is its own name (`bare-documents.md`). The other seeded documents keep
the explicit root, so the folder shows both forms — and note that this
paragraph cannot spell that root's tag, even in backticks, because hick has
no escaping and would read it as a tag. Documents that need to talk about
the syntax rebind the prefix to `h:` and keep their wrapper.

**What to check.** Scroll from the first line to the last and read the left
gutter. The numbers must run unbroken — no skipped number anywhere, whether a
block is showing its source or its result. Every annotation below rides the
END of a line that already has a number; the only rows allowed to stand for
more than one line are the two folds (a rendered cell, a rendered diagram),
and those step over lines that are genuinely not being shown.

<hick:feature name="extra" description="Shows the second when-banner instead of the first" />

## The environment, annotated inline

The declaration below is one line, and the executor note the app appends to it
is on that same line. Where the environment's commands actually run is the
only thing chrome can add here; the name, image, and rules are already source
text on numbered lines.

<hick:container name="shell" image="alpine:3.20" />
<hick:volume name="src" input="." />

## Cells — the rail's `▶` icon

Two of them, so the rail has to stack a pair of icons and the labels have to
count. The first names its container, so its icon reads `Cell 1 — shell`.

<hick:exec container="shell">
printf 'pear,4\nplum,2\npear,6\n' > fruit.csv
cat fruit.csv
<hick:expect match="exact">pear,4
plum,2
pear,6
</hick:expect>
</hick:exec>

The second is the cell a diagram points at, so it carries an `id`. A cell that
proves a picture is the one place an exec has to be findable by name.

<hick:exec id="no-back-edges" container="shell">
awk -F, '{sum[$1]+=$2} END {for (k in sum) print k, sum[k]}' fruit.csv | sort
<hick:expect match="exact">pear 10
plum 2
</hick:expect>
</hick:exec>

## A diagram — the rail's `◈` icon

Rendered by default and swapped back to source from the same icon. It asserts
the cell above, so the picture cannot quietly stop being true.

<hick:diagram renderer="mermaid" asserts="#no-back-edges">
flowchart TD
  csv[fruit.csv] --> awk[awk sum]
  awk --> totals[totals]
</hick:diagram>

## Prose fences — the rail's `≡` icon

A fence in prose is a command nobody wired up. Its icon offers to make it a
real cell. One with an info string:

```sh
sort -t, -k2 -n fruit.csv
```

And one without, because the converter has to guess the language for this one:

```
wc -l < fruit.csv
```

A fence inside verbatim payload is deliberately NOT a card — the one in the
generated file below is content, not a suggestion.

## A generated file — the path chip

<hick:file path="notes.txt">
Assembled from the fragments below.

<hick:paste select=".note" />
</hick:file>

## Fragments — the handle chips

A `copy` contributes to the file above and stays in the woven markdown:

<hick:copy id="note-first" class="note">
Cards ride the end of a line. Nothing here adds a row.
</hick:copy>

A `cut` contributes and then removes itself from the weave, which is why its
chip says `cut` rather than `copy`:

<hick:cut id="note-second" class="note">
This sentence reaches notes.txt and never reaches cards.md.
</hick:cut>

## Conditionals — the banners

Two `when` blocks, exactly one of which is live. Run with `--features extra`
to swap them.

<hick:when test="!extra">
The default section. Nothing was asked for, so this is what weaves.
</hick:when>

<hick:when test="extra">
The feature section. `--features extra` was passed.
</hick:when>

## The end

If the gutter counted straight from line 1 to here, the cards are behaving.
EOF

seed_file "$PROJECT_DIR/sessions/20260820-090000-every-turn-chip.hick" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-08-20T09:00:00Z">
<hick:user>Show me every turn chip at once — you, agent, tool, result, ran, output.</hick:user>
<hick:assistant>
A session is the other half of the card fixture: turn chips are the annotations
a `.hick` document cannot carry, because a session is its own root element.
Each one below rides the end of the line its element opens on.

<hick:tool name="read_doc">
<hick:input>cards.md</hick:input>
</hick:tool>
</hick:assistant>
<hick:tool-result name="read_doc" ok="true">
cards.md — 1 container, 2 cells, 1 diagram, 2 prose fences, 1 file, 2 fragments.
</hick:tool-result>
<hick:assistant>
Here is a tool the agent was refused, so the chip reads `refused` rather than
`ok`:

<hick:tool name="write_doc">
<hick:input>cards.md</hick:input>
</hick:tool>
</hick:assistant>
<hick:tool-result name="write_doc" ok="false">
declined — this document is a fixture and is edited by hand
</hick:tool-result>
<hick:assistant>
And a command, with its output. The `ran` chip names the language; the
`output` chip names the exit status, and says "failed" when it is not zero.

<hick:action lang="sh">
wc -l < fruit.csv
</hick:action>
</hick:assistant>
<hick:observation source="action-0" exit="0">
3
</hick:observation>
<hick:assistant>
A failing one, so the red half of the output chip is on screen too:

<hick:action lang="sh">
grep quince fruit.csv
</hick:action>
</hick:assistant>
<hick:observation source="action-1" exit="1">
</hick:observation>
<hick:user>Good — the gutter should still count straight through all of it.</hick:user>
</hick:session>
EOF

# Weave with a `hick` built from THIS checkout, so a fresh seed is CONSISTENT
# rather than drifted. Without this the first thing a developer might try —
# `hick test .dev/project` — reports a failure that is really just "nothing has
# run yet", which is a bad first impression of the drift gate.
#
# It builds rather than hunting for an artifact. The version this used to pick,
# `target/release/hick` ahead of `target/debug/hick`, meant a release binary
# from weeks ago beat a debug one from a minute ago and the scratch project got
# woven by code nobody was looking at. `cargo build` is a no-op when it is
# already current, so the cost of being right here is nothing.
if [ "$SEED_WEAVE" = "0" ]; then
  HICK=""
elif [ -z "${HICK:-}" ] && command -v cargo >/dev/null 2>&1; then
  echo
  echo "Building hick (no-op if it is already current)…"
  if cargo build --quiet --bin hick; then
    HICK=target/debug/hick
  fi
fi
[ -n "${HICK:-}" ] || [ "$SEED_WEAVE" = "0" ] || HICK="$(command -v hick || true)"

if [ -n "$HICK" ]; then
  echo
  echo "Weaving the seed with ${HICK}…"
  "$HICK" run "$PROJECT_DIR" >/dev/null 2>&1 || echo "  (weave skipped — the app will do it on open)"
fi

echo
if [ -n "$STALE" ]; then
  echo "Your copies of these files differ from the fixtures in git:"
  printf '%b' "$STALE" | sed 's/^/  /'
  echo
  echo "They were left exactly as you have them. If you did not mean to keep"
  echo "your version, delete the file and re-run \`just dev\` — it will be"
  echo "written fresh. \`just dev-clean\` does the same for the whole folder."
  echo
fi
echo "Seeded. \`just dev\` opens this folder; \`just dev-clean\` removes it."

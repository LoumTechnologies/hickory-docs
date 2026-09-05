#!/usr/bin/env bash
# No source file grows past a thousand lines, and the ones already past it
# only shrink.
#
# .instructions/continuous-integration.md asks for a file-length rule from
# the start; this repository reached 200k lines without one, and the cost is
# in docs/specs/freeform/the-minimal-core.md: a 4.4k-line main.rs, a 60k-line
# crate that is the architecture because nothing below it decides anything.
# A ratchet is the honest fix for a rule adopted late — the files over the
# line are named in scripts/file-length-baseline.txt with the length they
# had when the rule arrived, a file may never exceed its baseline, and a
# file not in the baseline may never exceed the threshold. Shrinking a file
# lowers its baseline (`--write`); nothing raises one.
#
# Called identically by `just check-file-length`, the pre-commit hook and CI.
set -euo pipefail
cd "$(dirname "$0")/.."

threshold=1000
baseline=scripts/file-length-baseline.txt
write=false
[ "${1:-}" = "--write" ] && write=true

# Tracked source only: generated files, tests and vendored trees are not
# what the rule is about.
current="$(
  git ls-files -z -- '*.rs' '*.ts' '*.tsx' \
    | tr '\0' '\n' \
    | grep -vE 'node_modules/|/generated/|\.test\.(ts|tsx)$|/tests/|\.d\.ts$' \
    | xargs -d '\n' wc -l \
    | awk -v t="$threshold" '$2 != "total" && $1 > t { print $2, $1 }' \
    | sort
)"

if $write; then
  printf '%s\n' "$current" > "$baseline"
  echo "wrote $baseline ($(printf '%s\n' "$current" | grep -c . || true) files over $threshold lines)"
  exit 0
fi

status=0
while read -r file lines; do
  [ -n "$file" ] || continue
  allowed=$(awk -v f="$file" '$1 == f { print $2 }' "$baseline")
  if [ -z "$allowed" ]; then
    echo "$file: $lines lines, over the $threshold-line limit and not in $baseline" >&2
    echo "  Split it. A new file does not get a baseline entry." >&2
    status=1
  elif [ "$lines" -gt "$allowed" ]; then
    echo "$file: $lines lines, grew past its baseline of $allowed" >&2
    echo "  A file over the limit may only shrink. Move what you added somewhere smaller." >&2
    status=1
  fi
done <<<"$current"

# A file that shrank, or fell under the limit, lowers its own baseline —
# the ratchet only turns one way, and it is not turned by hand.
stale=false
while read -r file allowed; do
  [ -n "$file" ] || continue
  now=$(awk -v f="$file" '$1 == f { print $2 }' <<<"$current")
  if [ -z "$now" ] || [ "$now" -lt "$allowed" ]; then
    stale=true
  fi
done <"$baseline"
if $stale; then
  echo "$baseline is behind: a file shrank. Run \`just check-file-length --write\` and commit it." >&2
  status=1
fi

exit "$status"

#!/usr/bin/env bash
# Fail if anything generated is out of date with the source it came from.
#
# Called identically by `just check-codegen`, the pre-commit hook, and CI, so
# the three can never disagree about what counts as stale — the rule in
# .instructions/pre-commit-ci-parity.md, and the reason `scripts/dist.sh`
# names this file as the pattern to follow.
#
# Pass --write to regenerate instead of checking; that is `just codegen`.
#
# Today there is one generated file: the web app's language table, emitted
# from `hick-lsp`'s routing table. It exists because the two were once
# written down separately and drifted apart in BOTH directions at once —
# nineteen file extensions' worth — which made `hick lang` report languages
# as highlighted that opened as grey text, and highlight languages that were
# routed nowhere at all.
set -euo pipefail

cd "$(dirname "$0")/.."

out=apps/web/src/editor/generated/languages.ts
write=false
[ "${1:-}" = "--write" ] && write=true

if $write; then
  mkdir -p "$(dirname "$out")"
  cargo run -q -p hick-lsp --bin emit-languages > "$out"
  echo "wrote $out"
  exit 0
fi

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT
cargo run -q -p hick-lsp --bin emit-languages > "$tmp"
if ! diff -u "$out" "$tmp"; then
  echo >&2
  echo "$out is stale. Run \`just codegen\` and commit the result." >&2
  exit 1
fi

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
# Two things are generated:
#
# 1. The web app's language table, emitted from `hick-lsp`'s routing table.
#    It exists because the two were once written down separately and drifted
#    apart in BOTH directions at once — nineteen file extensions' worth —
#    which made `hick lang` report languages as highlighted that opened as
#    grey text, and highlight languages that were routed nowhere at all.
#
# 2. The parser the editor runs: `hick-lang` compiled to WebAssembly
#    (`crates/hick-lang-wasm`), so the editor draws a document's structure
#    with the same parser `hick run` reads it with. The editor used to carry
#    a parser of its own and disagreed on rebound prefixes, verbatim
#    elements and documents about hick's own syntax. The build is
#    reproducible byte for byte on the pinned toolchain, which is what lets
#    the artifact be committed and checked like any other generated file.
#    Needs the `wasm32-unknown-unknown` target and `wasm-pack`
#    (docs/developers/developer-environment.md).
set -euo pipefail

cd "$(dirname "$0")/.."

langs=apps/web/src/editor/generated/languages.ts
wasm_dir=apps/web/src/editor/generated/hick-lang
write=false
[ "${1:-}" = "--write" ] && write=true

# Build the parser into `dir`, leaving only the four files the app imports.
build_wasm() {
  local dir="$1"
  if ! command -v wasm-pack >/dev/null 2>&1; then
    echo "wasm-pack is not installed; it builds the editor's parser." >&2
    echo "  cargo install wasm-pack --version 0.15.0 --locked" >&2
    echo "  rustup target add wasm32-unknown-unknown" >&2
    exit 1
  fi
  wasm-pack build crates/hick-lang-wasm --target web --release \
    --out-dir "$dir" --out-name hick_lang >/dev/null 2>&1 \
    || wasm-pack build crates/hick-lang-wasm --target web --release \
      --out-dir "$dir" --out-name hick_lang
  rm -f "$dir/.gitignore" "$dir/package.json" "$dir/README.md"
}

if $write; then
  mkdir -p "$(dirname "$langs")"
  cargo run -q -p hick-lsp --bin emit-languages > "$langs"
  echo "wrote $langs"
  build_wasm "$(pwd)/$wasm_dir"
  echo "wrote $wasm_dir"
  exit 0
fi

tmp="$(mktemp)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmp" "$tmpdir"' EXIT
status=0

cargo run -q -p hick-lsp --bin emit-languages > "$tmp"
if ! diff -u "$langs" "$tmp"; then
  echo >&2
  echo "$langs is stale. Run \`just codegen\` and commit the result." >&2
  status=1
fi

build_wasm "$tmpdir"
if ! diff -rq "$wasm_dir" "$tmpdir"; then
  echo >&2
  echo "$wasm_dir is stale. Run \`just codegen\` and commit the result." >&2
  status=1
fi

exit "$status"

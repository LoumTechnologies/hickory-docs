#!/usr/bin/env bash
# Verify the dev environment's scripts — the check `scripts/affected-checks.sh`
# selects when `scripts/dev*.sh` or the justfile changes.
#
# There are two failure modes worth a CI job here, and the second is why this
# file exists at all:
#
#   1. A shell bug. macOS ships bash 3.2, so `bash -n` is run over anything a
#      developer on a Mac executes — the same reason the release scripts are
#      syntax-checked in the desktop job.
#   2. A seed that shows you the wrong thing. `just dev` re-seeds on every run,
#      and the seed distinguishes a file you edited (kept, and reported) from a
#      file that is merely older than the fixture in git (replaced). Getting
#      that backwards either destroys work or silently shows a stale fixture,
#      and neither is visible until somebody is confused by their own screen.
#
# Runs against a throwaway directory, so it never touches .dev/.
# Usage: scripts/check-dev-seed.sh
set -euo pipefail
cd "$(dirname "$0")/.."

for script in scripts/dev.sh scripts/dev-seed.sh scripts/check-dev-seed.sh; do
  bash -n "$script"
done
echo "syntax ok: dev.sh, dev-seed.sh, check-dev-seed.sh"

ROOT=$(mktemp -d)
trap 'rm -rf "$ROOT"' EXIT
export HICKORY_SEED_ROOT="$ROOT"
export HICKORY_SEED_WEAVE=0

FIXTURE="$ROOT/project/cards.hick"

seed() {
  ./scripts/dev-seed.sh
}

expect() {
  local label="$1" pattern="$2" output="$3"
  if ! grep -q "$pattern" <<<"$output"; then
    echo "FAIL: $label" >&2
    echo "  expected a line matching: $pattern" >&2
    echo "  got:" >&2
    sed 's/^/    /' <<<"$output" >&2
    exit 1
  fi
  echo "ok: $label"
}

# 1. A fresh directory is seeded.
out=$(seed)
expect "a fresh directory is created" "created .*cards.hick" "$out"
[ -s "$FIXTURE" ] || { echo "FAIL: the fixture is empty" >&2; exit 1; }

# 2. Seeding again changes nothing and says so. This is the every-`just dev`
#    case, and it must be quiet about files it did not touch.
out=$(seed)
expect "an unchanged file is reported current" "current .*cards.hick" "$out"
if grep -q "kept .*cards.hick" <<<"$out"; then
  echo "FAIL: an untouched file was reported as edited" >&2
  exit 1
fi

# 3. A file you edited is kept, and named.
echo "a line I typed while developing" >>"$FIXTURE"
out=$(seed)
expect "an edited file is kept" "kept .*cards.hick" "$out"
expect "an edited file is listed at the end" "Your copies of these files differ" "$out"
grep -q "a line I typed while developing" "$FIXTURE" \
  || { echo "FAIL: an edit was destroyed" >&2; exit 1; }

# 4. A file that is merely OLD — matching what was seeded, not what the
#    fixture now says — is replaced. This is the case the old seed got wrong:
#    it wrote missing files only, so a changed fixture never reached a folder
#    that already existed.
stale="# an older version of this fixture"
printf '%s\n' "$stale" >"$FIXTURE"
if command -v sha256sum >/dev/null 2>&1; then
  hash=$(printf '%s\n' "$stale" | sha256sum | cut -d' ' -f1)
elif command -v shasum >/dev/null 2>&1; then
  hash=$(printf '%s\n' "$stale" | shasum -a 256 | cut -d' ' -f1)
else
  hash=$(printf '%s\n' "$stale" | openssl dgst -sha256 | awk '{print $NF}')
fi
grep -v " $ROOT/project/cards.hick\$" "$ROOT/seed-manifest" >"$ROOT/m" || true
echo "$hash $ROOT/project/cards.hick" >>"$ROOT/m"
mv "$ROOT/m" "$ROOT/seed-manifest"

out=$(seed)
expect "a stale-but-unedited file is replaced" "updated .*cards.hick" "$out"
if grep -q "an older version of this fixture" "$FIXTURE"; then
  echo "FAIL: a stale fixture survived a re-seed — \`just dev\` would show it" >&2
  exit 1
fi

# 5. A deleted file comes back.
rm "$FIXTURE"
out=$(seed)
expect "a deleted file is recreated" "created .*cards.hick" "$out"

# 6. The fixtures the seed writes are documents this build can actually parse.
#    A fixture that fails to parse turns `just dev` into a puzzle.
if [ -x target/debug/hick ] || [ -x target/release/hick ] || command -v hick >/dev/null 2>&1; then
  HICK=$(command -v hick || true)
  [ -x target/release/hick ] && HICK=target/release/hick
  [ -x target/debug/hick ] && HICK=target/debug/hick
  # `weave`, not `test`: weaving parses every document and writes its markdown
  # without executing a cell, which is exactly the question here. `test` would
  # report drift, and a seed that has never been run has drifted by definition.
  if ! weave_out=$("$HICK" weave "$ROOT/project" 2>&1); then
    echo "FAIL: a seeded fixture does not parse" >&2
    sed 's/^/  /' <<<"$weave_out" >&2
    exit 1
  fi
  echo "ok: the seeded fixtures parse and weave ($HICK)"
else
  echo "skip: no hick binary to parse the fixtures with"
fi

echo
echo "dev-seed: all checks passed."

<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain scripts/test-spatial-ui.sh

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-35e5d9040e28"></a>
## `scripts/test-spatial-ui.sh`

Related contracts: [Spatial Views Preserve The Editor](../../contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="scripts/test-spatial-ui.sh">
#!/usr/bin/env bash
# Native engine and drawer/editor geometry in a disposable workspace; no paid model.
set -euo pipefail
cd "$(dirname "$0")/.."
run_dir=$(mktemp -d "${TMPDIR:-/tmp}/hickory-spatial-ui.XXXXXX")
cleanup() {
  [ -z "${vite_pid:-}" ] || kill "$vite_pid" 2>/dev/null || true
  [ -z "${api_pid:-}" ] || kill "$api_pid" 2>/dev/null || true
  [ -z "${vite_pid:-}" ] || wait "$vite_pid" 2>/dev/null || true
  [ -z "${api_pid:-}" ] || wait "$api_pid" 2>/dev/null || true
  rm -rf "$run_dir"
}
trap cleanup EXIT
if [ -n "${HICKORY_SPATIAL_PROJECT:-}" ]; then
  cp -R "$HICKORY_SPATIAL_PROJECT" "$run_dir/project"
else
mkdir "$run_dir/project"
mkdir "$run_dir/project/notes"
printf '# First note\n\nA document with a real editor.\n\nMore lines to keep the selection visible.\n' > "$run_dir/project/note.md"
printf '# Related note\n\nAnother editable document.\n' > "$run_dir/project/notes/other.md"
fi
cargo build -p hickory-cli --example python_agent_ui_fixture
./target/debug/examples/python_agent_ui_fixture "$run_dir/project" > "$run_dir/api.log" 2> "$run_dir/api-error.log" &
api_pid=$!
for _ in $(seq 1 100); do
  [ ! -s "$run_dir/api.log" ] || break
  kill -0 "$api_pid" 2>/dev/null || { cat "$run_dir/api-error.log"; exit 1; }
  sleep .1
done
api_origin=$(head -1 "$run_dir/api.log")
[ -n "$api_origin" ] || { cat "$run_dir/api-error.log"; exit 1; }
HICKORY_API_ORIGIN="$api_origin" npm --prefix apps/web run dev > "$run_dir/vite.log" 2>&1 &
vite_pid=$!
ui_origin=""
for _ in $(seq 1 100); do
  ui_origin=$(sed -nE 's/.*Local: *(http:\/\/127.0.0.1:[0-9]+\/).*/\1/p' "$run_dir/vite.log" | head -1)
  [ -z "$ui_origin" ] || break
  kill -0 "$vite_pid" 2>/dev/null || { cat "$run_dir/vite.log"; exit 1; }
  sleep .1
done
[ -n "$ui_origin" ] || { cat "$run_dir/vite.log"; exit 1; }
[ "$#" -gt 0 ] || set -- spatialWorkspace.spec.ts
HICKORY_SPATIAL_URL="$ui_origin" HICKORY_E2E_CHANNEL=chromium HICKORY_E2E_URL="$ui_origin" \
  npm --prefix apps/web run test:e2e -- "$@"
</storycode:entry>

</storycode:doc>

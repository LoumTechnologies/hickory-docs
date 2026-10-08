#!/usr/bin/env bash
# Real browser, supervisor and confined worker in a disposable workspace; no paid model.
set -euo pipefail
cd "$(dirname "$0")/.."
run_dir=$(mktemp -d "${TMPDIR:-/tmp}/hickory-python-ui.XXXXXX")
cleanup() {
  [ -z "${vite_pid:-}" ] || kill "$vite_pid" 2>/dev/null || true
  [ -z "${api_pid:-}" ] || kill "$api_pid" 2>/dev/null || true
  [ -z "${vite_pid:-}" ] || wait "$vite_pid" 2>/dev/null || true
  [ -z "${api_pid:-}" ] || wait "$api_pid" 2>/dev/null || true
  rm -rf "$run_dir"
}
trap cleanup EXIT
mkdir "$run_dir/project"
printf '# Original human input\n' > "$run_dir/project/note.md"
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
HICKORY_AGENT_TEST_PYTHON="$PWD/apps/agent/.venv/bin/python" HICKORY_PYTHON_UI_FIXTURE=1 HICKORY_E2E_CHANNEL=chromium HICKORY_E2E_URL="$ui_origin" \
  npm --prefix apps/web run test:e2e -- python-agent.spec.ts

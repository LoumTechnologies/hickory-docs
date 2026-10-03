#!/usr/bin/env bash
# A real engine and browser against an isolated Git repository; no existing app is touched.
set -euo pipefail
cd "$(dirname "$0")/.."
run_dir=$(mktemp -d "${TMPDIR:-/tmp}/hickory-literate-e2e.XXXXXX")
cleanup() {
  [ -z "${vite_pid:-}" ] || kill "$vite_pid" 2>/dev/null || true
  [ -z "${client_pid:-}" ] || kill "$client_pid" 2>/dev/null || true
  [ -z "${client_pid:-}" ] || wait "$client_pid" 2>/dev/null || true
  [ -z "${vite_pid:-}" ] || wait "$vite_pid" 2>/dev/null || true
  if [ -f "$run_dir/state/engine/endpoint.json" ]; then
    python3 - "$run_dir/state/engine/endpoint.json" <<'PY'
import json, os, signal, sys
try: os.kill(json.load(open(sys.argv[1]))['pid'], signal.SIGTERM)
except (ProcessLookupError, KeyError): pass
PY
  fi
  rm -rf "$run_dir"
}
trap cleanup EXIT
mkdir -p "$run_dir/project" "$run_dir/state"
printf 'value = 1\n' > "$run_dir/project/code.py"
printf '.hick-cache/\n' > "$run_dir/project/.gitignore"
git -C "$run_dir/project" init -q
git -C "$run_dir/project" -c user.name=Test -c user.email=tests@example.invalid add .
git -C "$run_dir/project" -c user.name=Test -c user.email=tests@example.invalid commit -qm initial
for revision in $(seq 1 6); do
  printf '%s\n' "$revision" > "$run_dir/project/revision.txt"
  git -C "$run_dir/project" add revision.txt
  git -C "$run_dir/project" -c user.name=Test -c user.email=tests@example.invalid commit -qm "revision $revision"
done
ui_port=$(python3 - <<'PY'
import socket
with socket.socket() as s:
    s.bind(('127.0.0.1',0)); print(s.getsockname()[1])
PY
)
HICKORY_STATE_DIR="$run_dir/state" target/debug/examples/engine_client "$run_dir/project" literate-e2e "http://127.0.0.1:$ui_port" > "$run_dir/client.log" 2> "$run_dir/engine.log" &
client_pid=$!
for _ in $(seq 1 100); do
  [ ! -s "$run_dir/client.log" ] || break
  kill -0 "$client_pid" 2>/dev/null || { cat "$run_dir/engine.log"; exit 1; }
  sleep .1
done
api_origin=$(head -1 "$run_dir/client.log")
[ -n "$api_origin" ] || { cat "$run_dir/engine.log"; exit 1; }

HICKORY_API_ORIGIN="$api_origin" npm --prefix apps/web run dev -- --port "$ui_port" --strictPort > "$run_dir/vite.log" 2>&1 &
vite_pid=$!
for _ in $(seq 1 100); do
  curl -fsS "http://127.0.0.1:$ui_port/" >/dev/null 2>&1 && break
  kill -0 "$vite_pid" 2>/dev/null || { cat "$run_dir/vite.log"; exit 1; }
  sleep .1
done
HICKORY_E2E_CHANNEL=chromium HICKORY_E2E_URL="http://127.0.0.1:$ui_port" npm --prefix apps/web run test:e2e -- literate-editor.spec.ts

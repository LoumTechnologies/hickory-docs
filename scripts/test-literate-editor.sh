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
awk 'BEGIN { print "Managed startup text"; for (line = 1; line <= 180; line++) printf "Managed startup row %03d\n", line }' > "$run_dir/project/managed startup #é.md"
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
if [ -n "${HICKORY_THREAD_TEST_VENV:-}" ]; then ln -s "$HICKORY_THREAD_TEST_VENV" "$run_dir/project/.venv"; fi
if [ -n "${HICKORY_RECORDED_E2E:-}" ]; then
  : "${HICKORY_THREAD_TEST_VENV:?Choose a local Python environment with debugpy installed}"
  test -x "$HICKORY_THREAD_TEST_VENV/bin/python"
  export PATH="$HICKORY_THREAD_TEST_VENV/bin:$PATH"
  mkdir -p "$run_dir/project/sessions"
  cat > "$run_dir/project/sessions/agent.md" <<'SESSION'
<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-11-18T10:00:00Z" doc=".hick-workspace-agent">
<hick:user id="u1" turn="main-recorded-debug" provider="codex-subscription" model="test">Recorded request.

```python
value = 41
print(value + 1)
```

```python
raise RuntimeError('wrong user fence')
```
</hick:user>
<hick:assistant>Recorded answer.

~~~python
value = 52
print(value + 1)
~~~

~~~python
raise RuntimeError('wrong assistant fence')
~~~
</hick:assistant>
<hick:context kind="acp-turn-status">{"turn":"main-recorded-debug","status":"ok"}</hick:context>
</hick:session>
SESSION
  export HICKORY_RECORDED_E2E_ROOT="$run_dir/project"
fi
if [ -n "${HICKORY_RECORDED_PATCH_E2E:-}" ]; then
  mkdir -p "$run_dir/project/sessions"
  cat > "$run_dir/project/project.md" <<'PROJECT'
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:project sources="author.md"/>
</hick:doc>
PROJECT
  cat > "$run_dir/project/author.md" <<'AUTHOR'
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="generated.py">
value = 1
</hick:file>
</hick:doc>
AUTHOR
  printf 'value = 1\n' > "$run_dir/project/generated.py"
  printf 'saved note\n' > "$run_dir/project/review.txt"
  cat > "$run_dir/project/sessions/agent.md" <<'PATCH_SESSION'
<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-11-18T10:00:00Z" doc=".hick-workspace-agent">
<hick:user id="u1" turn="main-recorded-patch" provider="codex-subscription" model="test">Review this construction.

```diff
--- a/author.md
+++ b/author.md
@@ -3 +3 @@
-value = 1
+value = 41
```
</hick:user>
<hick:assistant>Review this note.

```patch
--- a/review.txt
+++ b/review.txt
@@ -1 +1 @@
-saved note
+assistant draft
```
</hick:assistant>
<hick:context kind="acp-turn-status">{"turn":"main-recorded-patch","status":"ok"}</hick:context>
</hick:session>
PATCH_SESSION
  if [ -n "${HICKORY_RECORDED_PATCH_MULTI:-}" ]; then
    python3 - "$run_dir/project/sessions/agent.md" "$HICKORY_RECORDED_PATCH_MULTI" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
original = path.read_text()
if sys.argv[2] == "assistant":
    side = "assistant"
    addition = "--- a/author.md\n+++ b/author.md\n@@ -3 +3 @@\n-value = 1\n+value = 53\n"
else:
    side = "user"
    addition = "--- a/review.txt\n+++ b/review.txt\n@@ -1 +1 @@\n-saved note\n+user draft\n"
marker = chr(96) * 3 + f"\n</hick:{side}>"
assert original.count(marker) == 1
path.write_text(original.replace(marker, addition + marker, 1))
PY
  fi
  export HICKORY_RECORDED_PATCH_ROOT="$run_dir/project"
fi
if [ -n "${HICKORY_RECORDED_CONSTRUCTION_E2E:-}" ]; then
  python3 - "$run_dir/project" "$HICKORY_RECORDED_CONSTRUCTION_E2E" <<'PY'
from pathlib import Path
import runpy
import sys

side = "user" if sys.argv[2] == "user" else "assistant"
runpy.run_path("scripts/test-simple-browser.py")["prepare_recorded_construction"](Path(sys.argv[1]), side)
PY
  export HICKORY_RECORDED_CONSTRUCTION_ROOT="$run_dir/project"
fi
HICKORY_STATE_DIR="$run_dir/state" "${HICKORY_LITERATE_TEST_ENGINE:-target/debug/examples/engine_client}" "$run_dir/project" literate-e2e "http://127.0.0.1:$ui_port" > "$run_dir/client.log" 2> "$run_dir/engine.log" &
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
HICKORY_ACP_FIXTURE="$(pwd)/target/debug/examples/acp_fixture" HICKORY_E2E_CHANNEL=chromium HICKORY_E2E_URL="http://127.0.0.1:$ui_port" npm --prefix apps/web run test:e2e -- literate-editor.spec.ts "$@"

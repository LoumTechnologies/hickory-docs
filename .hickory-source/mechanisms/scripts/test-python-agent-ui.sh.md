<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain scripts/test-python-agent-ui.sh

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9c041bcaa17d"></a>
## `scripts/test-python-agent-ui.sh`

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="scripts/test-python-agent-ui.sh" executable="true">
#!/usr/bin/env bash
# Real browser, supervisor and confined worker in a disposable workspace.
# Scripted by default; --live-provider explicitly enables paid provider calls.
set -euo pipefail
cd "$(dirname "$0")/.."
run_dir=$(mktemp -d "${TMPDIR:-/tmp}/hickory-python-ui.XXXXXX")
fixture_args=()
test_file=python-agent.spec.ts
case "${1:-}" in
  "") ;;
  --attachments) export HICKORY_ATTACHMENT_UI_FIXTURE=1 ;;
  --live-provider)
    fixture_args=(--live-provider)
    test_file=liveAgentInvestigation.spec.ts
    export HICKORY_LIVE_PROVIDER_WITNESS=1
    ;;
  *) echo 'Usage: scripts/test-python-agent-ui.sh [--live-provider|--attachments]' >&2; exit 2 ;;
esac
[ "$#" -le 1 ] || { echo 'Unexpected arguments' >&2; exit 2; }
terminate_tree() {
  local parent="$1" child
  while read -r child; do
    [ -z "$child" ] || terminate_tree "$child"
  done < <(pgrep -P "$parent" || true)
  kill "$parent" 2>/dev/null || true
}
cleanup() {
  [ -z "${vite_pid:-}" ] || terminate_tree "$vite_pid"
  [ -z "${api_pid:-}" ] || terminate_tree "$api_pid"
  [ -z "${vite_pid:-}" ] || wait "$vite_pid" 2>/dev/null || true
  [ -z "${api_pid:-}" ] || wait "$api_pid" 2>/dev/null || true
  if [ "${HICKORY_KEEP_UI_WITNESS:-0}" = 1 ]; then
    echo "Private UI witness retained at $run_dir; review before sharing."
  else
    rm -rf "$run_dir"
  fi
}
trap cleanup EXIT
mkdir "$run_dir/project"
printf '# Original human input\n' > "$run_dir/project/note.md"
if [ "$test_file" = liveAgentInvestigation.spec.ts ]; then
  python3 - "$run_dir/project" <<'PY'
from pathlib import Path
import sys
root = Path(sys.argv[1])
for source, target in [('examples/ai-engineer-deadline.md', 'deadline-spec.md'), ('examples/deadline.js', 'deadline.js')]:
    text = Path(source).read_text().replace('var allowed = Math.max(0, budget - total);', 'var allowed = budget;')
    (root / target).write_text(text)
(root / 'AGENTS.md').write_text('The request deadline is a total budget. Retain short-attempt and empty-sequence checks.\n')
(root / '.gitignore').write_text('sessions/\n.hick-cache/\n.hick/\nui.json\n')
PY
  git -C "$run_dir/project" init -q --initial-branch=master
  git -C "$run_dir/project" add .
  git -C "$run_dir/project" -c user.name='Hickory synthetic fixture' -c user.email=fixture@hickory.invalid -c commit.gpgsign=false -c core.hooksPath=/dev/null commit -qm 'Seed synthetic deadline fixture'
fi
cargo build -p hickory-cli --example python_agent_ui_fixture
"${CARGO_TARGET_DIR:-target}/debug/examples/python_agent_ui_fixture" "$run_dir/project" "${fixture_args[@]}" > "$run_dir/api.log" 2> "$run_dir/api-error.log" &
api_pid=$!
for _ in $(seq 1 100); do
  [ ! -s "$run_dir/api.log" ] || break
  kill -0 "$api_pid" 2>/dev/null || { cat "$run_dir/api-error.log"; exit 1; }
  sleep .1
done
api_origin=$(head -1 "$run_dir/api.log")
[ -n "$api_origin" ] || { cat "$run_dir/api-error.log"; exit 1; }
# Exercise attachment refusal through the real HTTP boundary before any model
# request. These cases must fail even without a configured Python runtime.
python3 - "$api_origin" <<'PYATTACH'
import json, sys, urllib.request, urllib.error
origin = sys.argv[1]
for attachment in [
    dict(name='fake.png', media_type='image/png', data='aGVsbG8='),
    dict(name='bad.txt', media_type='text/plain', data='!not-base64!'),
    dict(name='x' * 256, media_type='text/plain', data='aA=='),
    dict(name='large.txt', media_type='text/plain', data='A' * 524289),
]:
    payload = dict(backend='python', prompt='Do not run', context=dict(attachments=[attachment]))
    request = urllib.request.Request(origin + '/api/docs/workspace/agent',
        data=json.dumps(payload).encode(), headers={'Content-Type': 'application/json'})
    try:
        urllib.request.urlopen(request)
        raise AssertionError('Invalid attachment was accepted')
    except urllib.error.HTTPError as error:
        assert error.code == 400, (error.code, error.read())
print('Attachment HTTP safety checks passed')
PYATTACH
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
HICKORY_E2E_OUTPUT_DIR="$run_dir/browser" npm --prefix apps/web run test:e2e -- "$test_file"
</storycode:entry>

</storycode:doc>

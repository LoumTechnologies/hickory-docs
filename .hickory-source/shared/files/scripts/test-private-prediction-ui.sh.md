<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# scripts/test-private-prediction-ui.sh

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../guarantees/verification/predictions-precede-managed-observation/contract.md)
- [Private edit proposals open review before writing](../../../guarantees/verification/private-proposals-wait-for-review/contract.md)

Captured SHA-256: `23ea3addb49e34c756b137123d95734eb1ade09a5bcac9c517b5c8aa27c0a678`.

<book:entry path="scripts/test-private-prediction-ui.sh" executable="true">
#!/usr/bin/env bash
# Focused actual engine/worker/browser witness with a local scripted provider.
set -euo pipefail
cd "$(dirname "$0")/.."
live=0
acp=0
case "${1:-}" in "") ;; --live-provider) live=1 ;; --acp-fixture) acp=1 ;; --live-acp) live=1; acp=1; [ -n "${HICKORY_PRIVATE_ACP_CONFIG:-}" ] || { echo 'Set HICKORY_PRIVATE_ACP_CONFIG to an explicit private-qa ACP profile' >&2; exit 2; } ;; *) echo 'Usage: scripts/test-private-prediction-ui.sh [--live-provider|--acp-fixture|--live-acp]' >&2; exit 2 ;; esac
[ "$#" -le 1 ] || exit 2
# The worker is a non-editable installed wheel. Refresh it before every witness,
# otherwise a source fix can silently exercise the previous installed policy.
# Reuse the configured interpreter: it is the one actually witnessed, even when
# this host lacks the repository's preferred Python download.
uv sync --project apps/agent --locked --no-editable --reinstall-package hickory-agent-worker \
  --python "$PWD/apps/agent/.venv/bin/python"
run_dir=$(mktemp -d "${TMPDIR:-/tmp}/hickory-private-ui.XXXXXX")
terminate_tree() {
  local parent="$1" child
  while read -r child; do [ -z "$child" ] || terminate_tree "$child"; done < <(pgrep -P "$parent" || true)
  kill "$parent" 2>/dev/null || true
}
cleanup() {
  for pid in "${vite_pid:-}" "${api_pid:-}" "${provider_pid:-}"; do
    [ -z "$pid" ] || terminate_tree "$pid"
  done
  # This isolated daemon belongs only to the fixture; never touch the user's engine.
  if [ -f "$run_dir/state/engine/endpoint.json" ]; then
    python3 - "$run_dir/state/engine/endpoint.json" <<'PY'
import json,os,signal,sys
try: os.kill(json.load(open(sys.argv[1]))['pid'],signal.SIGTERM)
except (ProcessLookupError,FileNotFoundError): pass
PY
  fi
  if [ "${HICKORY_KEEP_UI_WITNESS:-0}" = 1 ]; then
    echo "Private fixture retained at $run_dir; this is not reviewed publication footage."
  else rm -rf "$run_dir"; fi
}
trap cleanup EXIT
mkdir -p "$run_dir/project" "$run_dir/ledger" "$run_dir/state"
python3 - "$run_dir" "$live" <<'PY'
import hashlib,json,sys
from pathlib import Path
root=Path(sys.argv[1]); digest=lambda value:hashlib.sha256(value.encode()).hexdigest()
source=Path('examples/ai-engineer-private-deadline.md').read_text()
corrected=source.replace('allowance = budget','allowance = budget - elapsed')
(root/'project/doc.md').write_text(source)
adapter=Path('.hick-cache/adapters/python').resolve()
if not (adapter/'bin/python').exists(): raise SystemExit('Install the configured Python DAP adapter before this focused witness')
(root/'project/.hick-cache/adapters').mkdir(parents=True)
(root/'project/.hick-cache/adapters/python').symlink_to(adapter,target_is_directory=True)
start=source.index('        elapsed +='); end=source.index('\n',start)+1
output={'kind':'output','execution':f'c:{source[:source.index(chr(60)+"hick:exec")].count(chr(10))+1}:json'}
target={'kind':'combined','output':output,'debugger':{'kind':'debugger','source':'root','source_revision':digest(source),'start_byte':start,'end_byte':end,'expression':'allowance','hit':2,'frame':'retry'}}
requirement={'kind':'fields','value':{'output':{'kind':'fields','value':{'long':{'kind':'number_at_most','value':100},'short':{'kind':'equals','value':30},'empty':{'kind':'equals','value':0}}},'debugger':{'kind':'number_at_most','value':20}}}
predicted={'kind':'fields','value':{'output':{'kind':'equals','value':{'long':100,'short':30,'empty':0}},'debugger':{'kind':'equals','value':20}}}
profile={'ledger_root':str(root/'ledger'),'seal':'','request_id':'initial-unsealed','identity':{'source':digest(source),'inputs':digest('[[80,80],[10,20],[]]'),'dependencies':{},'environment':{'fixture-host':digest('declared-local-python-and-configured-debugpy-not-frozen')}},'target':target,'actor':{'kind':'agent','id':'private-browser-predictor'},'encoding':'json','question':{'family':'private-browser-budget','inputs':[[80,80],[10,20],[]],'requirement_reference':'One total budget of 100ms; preserve short and empty cases and the second-attempt allowance','requirement':requirement,'identity_known':True}}
profile['retained']={'initial':{'family':'private-browser-initial','inputs':profile['question']['inputs'],'requirement_reference':'Initial allowance is the full 100ms total budget','requirement':{'kind':'equals','value':100},'identity_known':True},'next':{'family':'private-browser-next','inputs':profile['question']['inputs'],'requirement_reference':'The second allowance is 20ms after the first 80ms attempt','requirement':{'kind':'equals','value':20},'identity_known':True}}
profile['runtime']={'kind':'python_debugpy','project':str(root/'project')}
profile['inputs']={'value':profile['question']['inputs'],'anchors':[next(line for line in source.splitlines() if line.startswith('print(json.dumps('))]}
disclosure='Configured real-provider mode; review decisions are made by the test driver.' if sys.argv[2]=='1' else 'Scripted provider and test-driver review decisions.'
profile['question']['requirement_reference']='Synthetic source and inputs; actual output/DAP comparisons. '+disclosure+' '+profile['question']['requirement_reference']
(root/'profile.json').write_text(json.dumps(profile))
calls=[]
def tool(name,args):
 calls.append([name,args]);return json.dumps({'python':f'import json\ntool({name!r}, json.loads({json.dumps(args)!r}))'})
def seal(text):return tool('seal_prediction',{'expected_source_hash':digest(text),'predicted':predicted,'rationale':'The unchanged total budget bounds both attempts'})
def retained(text):
 return [tool('seal_debugger_prediction',{'expected_source_hash':digest(text),'predicted':{'kind':'equals','value':100},'rationale':'Before launch, the initial allowance is the full total budget'}),tool('continue_debugger_prediction',{}),tool('seal_debugger_prediction',{'expected_source_hash':digest(text),'predicted':{'kind':'equals','value':20},'rationale':'Before Continue, subtract the first 80ms attempt from the 100ms total'}),tool('continue_debugger_prediction',{})]
actions=[tool('ui_observe',{'parent':'root'}),tool('ui_observe',{'parent':'private-prediction'}),tool('read_doc',{}),*retained(source),seal(source),tool('verify',{}),tool('edit_doc',{'snapshot':digest(source),'whole':'true','input':corrected}),*retained(corrected),seal(corrected),tool('verify',{}),json.dumps({'done':'Reviewed correction passed the unchanged initial/future debugger and output predictions; current certification remains unavailable.'})]
(root/'actions.json').write_text(json.dumps(actions))
(root/'acp-tools.json').write_text(json.dumps(calls))
(root/'agents').mkdir()
(root/'agents/agents.json').write_text(json.dumps([{'id':'private-qa','name':'Scripted private ACP witness','command':sys.executable,'args':[],'workspace_filesystem':False,'private_prediction_args':[str(Path('scripts/fixtures/private-acp.py').resolve()),str(root/'acp-tools.json')]}]))
PY
if [ "$live" = 1 ] && [ "$acp" = 1 ]; then
  # Explicit trusted QA profile only; never reuse the user's ordinary arguments.
  python3 - "$HICKORY_PRIVATE_ACP_CONFIG" "$run_dir/agents/agents.json" <<'PYACP'
import json,sys
from pathlib import Path
profile=json.loads(Path(sys.argv[1]).read_text())
assert len(profile)==1 and profile[0]['id']=='private-qa'
assert profile[0].get('workspace_filesystem') is False
assert isinstance(profile[0].get('private_prediction_args'),list)
Path(sys.argv[2]).write_text(json.dumps(profile))
PYACP
fi
node --input-type=module - "$run_dir" > "$run_dir/provider.log" <<'JS' &
import {createServer} from 'node:http';
import {readFileSync} from 'node:fs';
const root=process.argv[2],actions=JSON.parse(readFileSync(`${root}/actions.json`));
const server=createServer(async(req,res)=>{for await(const _ of req){};const raw=actions.shift();if(!raw){res.writeHead(500);res.end('Scripted fixture exhausted');return;}res.writeHead(200,{'content-type':'text/event-stream'});res.end(`data: ${JSON.stringify({choices:[{delta:{content:raw},finish_reason:'stop'}]})}\n\ndata: [DONE]\n\n`);});
server.listen(0,'127.0.0.1',()=>console.log(`http://127.0.0.1:${server.address().port}/`));
JS
provider_pid=$!
for _ in $(seq 1 100); do [ ! -s "$run_dir/provider.log" ] || break; sleep .1; done
ui_port=$(python3 - <<'PYPORT'
import socket
with socket.socket() as s:
 s.bind(('127.0.0.1',0)); print(s.getsockname()[1])
PYPORT
)
ui_origin="http://127.0.0.1:$ui_port/"
cargo build -p hickory-cli --example engine_client --quiet
provider_env=(OPENAI_API_KEY=synthetic-fixture-key "OPENAI_BASE_URL=$(head -1 "$run_dir/provider.log")")
if [ "$live" = 1 ]; then provider_env=(HICKORY_TEST_LIVE_PROVIDER=1); fi
env HICKORY_STATE_DIR="$run_dir/state" HICKORY_TEST_PRIVATE_PROFILE="$run_dir/profile.json" \
HICKORY_AGENT_TEST_UI_SETTINGS="$run_dir/ui.json" HICKORY_AGENT_TEST_PYTHON="$PWD/apps/agent/.venv/bin/python" "${provider_env[@]}" \
target/debug/examples/engine_client "$run_dir/project/doc.md" private "${ui_origin%/}" > "$run_dir/api.log" 2> "$run_dir/api-error.log" &
api_pid=$!
for _ in $(seq 1 150); do [ ! -s "$run_dir/api.log" ] || break; kill -0 "$api_pid" 2>/dev/null || { cat "$run_dir/api-error.log"; exit 1; }; sleep .1; done
touch "$run_dir/vite.log"
HICKORY_API_ORIGIN="$(head -1 "$run_dir/api.log")" npm --prefix apps/web run dev -- --port "$ui_port" --strictPort > "$run_dir/vite.log" 2>&1 &
vite_pid=$!
ui_origin=""
for _ in $(seq 1 100); do
  ui_origin=$(sed -nE 's/.*Local: *(http:\/\/127.0.0.1:[0-9]+\/).*/\1/p' "$run_dir/vite.log" | head -1)
  [ -z "$ui_origin" ] || break
  kill -0 "$vite_pid" 2>/dev/null || { cat "$run_dir/vite.log"; exit 1; }; sleep .1
done
[ -n "$ui_origin" ] || { cat "$run_dir/vite.log"; exit 1; }
if [ "${HICKORY_PRIVATE_MANUAL_UI:-0}" = 1 ]; then
  # The same isolated fixture, driven manually instead of by Playwright.
  # The user's engine/executor policy is never changed.
  echo "Private QA UI: $ui_origin"
  echo "Scripted provider; isolated local executor. Stop this process to clean up."
  wait "$api_pid"
  exit 0
fi
HICKORY_PRIVATE_UI_FIXTURE=1 HICKORY_PRIVATE_ACP_FIXTURE="$acp" HICKORY_PRIVATE_LIVE_PROVIDER="$live" HICKORY_E2E_CHANNEL=chromium HICKORY_E2E_URL="$ui_origin" \
HICKORY_E2E_OUTPUT_DIR="$run_dir/browser" npm --prefix apps/web run test:e2e -- privatePredictionCoach.spec.ts

[ ! -e "$run_dir/project/deadline.py" ]
</book:entry>

</book:doc>

"""Owned engine/ACP/browser: sealed proposal review, whole draft Apply and resource Save."""
import argparse,hashlib,importlib.util,json,os,signal,socket,subprocess,sys,tempfile,threading,time,urllib.error,urllib.parse,urllib.request,uuid
from contextlib import contextmanager
from http.client import HTTPConnection
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def prepare(root):
 root.mkdir(parents=True,exist_ok=True);root=root.resolve()
 if any(root.iterdir()):raise ValueError('Agent proposal fixture requires an empty owned workspace')
 files={'project.md':'<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:project sources="construction.md"/>\n</h:doc>\n','construction.md':'<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="generated.py">\ngenerated = 1\n</h:file>\n</h:doc>\n','generated.py':'generated = 1\n','note.py':'value=1\n','behavior.md':'# Declared source behavior\n\nThe ordinary source may be proposed together with an explanation. Claims are not executed verification.\n'}
 for path,text in files.items():(root/path).write_text(text)
 (root/'hickory-dogfood').mkdir();(root/'hickory-dogfood/behavior-map.json').write_text(json.dumps({'format':1,'contracts':{'note':{'page':'behavior.md','title':'Declared source behavior','disposition':'retained-requirement','references':['note.py']}}}));(root/'hickory-dogfood/narrative-map.json').write_text(json.dumps({'format':1,'fragments':[]}));(root/'.hickory-native-agent-proposal-fixture').write_text('owned-native-agent-proposal-v1\n');subprocess.run(['git','init','-q',str(root)],check=True)
 return {'workspace':str(root),'entry':'project.md','source':'note.py','construction':'construction.md','generated':'generated.py','before':files['note.py'],'after':'value=41\n','marker':str(root/'.hickory-native-agent-proposal-fixture')}
def api(origin,method,path,body=None):
 request=urllib.request.Request(origin+path,data=None if body is None else json.dumps(body).encode(),headers={'Content-Type':'application/json'},method=method)
 try:
  with urllib.request.urlopen(request,timeout=90) as response:return json.load(response)
 except urllib.error.HTTPError as error:raise AssertionError(f'{method} {path} returned {error.code}: '+error.read().decode()) from error
def intention(edit):
 return {'version':1,'request_id':uuid.uuid4().hex,'origin':'controlled provider caller claim','title':'Explain and change the source together','explanation':'Propose the complete source transition and retain its declared behavior, generated consequences and evidence. This scripted provider makes no semantic test judgment.','claims':['Agent claims this should yield 41; this is unverified agent text.'],'check_ids':[],'capture':{'context':uuid.uuid4().hex,'project_entry':'project.md','drafts':{},'ownership_sources':{},'unaddressed_sources':{},'saved_guards':{},'edit':edit},'reading':{'version':1,'entry':'project.md','sources':{},'missing_sources':[],'focus':'note.py','unaddressed_sources':{},'saved_guards':{}}}
def provider_fixture(folder):
 calls=folder/'calls.json';result=folder/'tool-result.json';program=folder/'controlled-acp.py';settings=folder/'config/ui.json';agents=settings.parent/'agents';agents.mkdir(parents=True)
 program.write_text('''import json,sys,urllib.request
url=None
for line in sys.stdin:
 message=json.loads(line)
 if 'id' not in message:continue
 method=message.get('method')
 if method=='initialize':value={'protocolVersion':1,'agentCapabilities':{'mcpCapabilities':{'http':True}}}
 elif method=='session/new':url=message['params']['mcpServers'][0]['url'];value={'sessionId':'owned-proposal-provider'}
 elif method=='session/prompt':
  request=urllib.request.Request(url,json.dumps({'jsonrpc':'2.0','id':1,'method':'tools/call','params':{'name':'propose_workspace_changes','arguments':json.load(open(sys.argv[1]))}}).encode(),{'Content-Type':'application/json'})
  with urllib.request.urlopen(request,timeout=90) as response:outcome=json.load(response)
  json.dump(outcome,open(sys.argv[2],'w'))
  if outcome.get('result',{}).get('isError',True):raise RuntimeError(str(outcome))
  value={'stopReason':'end_turn'}
 elif method=='session/cancel':value={}
 else:raise RuntimeError('Unexpected controlled ACP method '+str(method))
 print(json.dumps({'jsonrpc':'2.0','id':message['id'],'result':value}),flush=True)
''')
 (agents/'agents.json').write_text(json.dumps([{'id':'owned-proposal','name':'Owned scripted proposal provider','command':sys.executable,'args':[str(program),str(calls),str(result)],'workspace_filesystem':False}]))
 return settings,calls,result
def controlled_tool(origin,calls,result,captured,edit):
 selected=intention(edit);calls.write_text(json.dumps({'intention':selected}));result.unlink(missing_ok=True)
 api(origin,'POST','/api/docs/workspace/agent/acp',{'backend':'owned-proposal'})
 api(origin,'POST','/api/docs/workspace/agent',{'backend':'owned-proposal','prompt':'Prepare the owned coherent intention; do not apply, save or run it.','context':captured})
 deadline=time.monotonic()+30
 while time.monotonic()<deadline:
  if result.exists():
   try:
    reply=json.loads(result.read_text());assert reply.get('result',{}).get('isError') is False,reply;summary=json.loads(reply['result']['content'][0]['text']);assert summary['pending_review'] and not summary['applied'] and not summary['saved'] and not summary['checks_executed'];return summary
   except json.JSONDecodeError:pass
  time.sleep(.05)
 raise AssertionError('Controlled ACP tool did not retain a pending coherent proposal')
@contextmanager
def trace_appkit_send(origin, evidence):
    upstream = urllib.parse.urlsplit(origin)
    lock = threading.Lock()
    hop = {"connection", "host", "transfer-encoding", "content-length", "keep-alive", "upgrade"}

    def trace(row):
        with lock, evidence.open("a") as output:
            output.write(json.dumps(row) + "\n")

    class Relay(BaseHTTPRequestHandler):
        def forward(self):
            size = int(self.headers.get("Content-Length", "0"))
            if not 0 <= size <= 256 * 1024:
                self.send_error(413, "Request exceeds the owned fixture limit")
                return
            data = self.rfile.read(size)
            path = urllib.parse.urlsplit(self.path).path
            selected = self.command == "POST" and path == "/api/docs/workspace/agent"
            report = self.command == "POST" and path == "/api/agent-proposals/report"
            original = json.loads(data) if selected or report else None
            forwarded = original
            if selected:
                if original.get("backend") != "python" or original.get("prompt") != PROMPT:
                    trace({"method": self.command, "path": self.path, "refused": True})
                    self.send_error(400, "Only the exact owned native Send fixture is routed")
                    return
                # The only change to the app's request prevents subscription use.
                forwarded = {**original, "backend": "owned-proposal"}
                data = json.dumps(forwarded).encode()
            connection = HTTPConnection(upstream.hostname, upstream.port, timeout=30)
            try:
                headers = {key:value for key,value in self.headers.items() if key.lower() not in hop}
                connection.request(self.command, self.path, body=data if data else None, headers=headers)
                response = connection.getresponse()
                row = {"method": self.command, "path": self.path, "status": response.status}
                streamed = "text/event-stream" in (response.getheader("Content-Type") or "")
                body = None if streamed else response.read()
                if path == "/api/files" and body is not None:
                    tree = json.loads(body)
                    row["tree_root"] = tree.get("root_path") if isinstance(tree, dict) else "invalid response"
                if selected or report:
                    row["request"] = original
                    row["response"] = json.loads(body)
                if selected:
                    row["fixture_backend_translation"] = [original["backend"], forwarded["backend"]]
                trace(row)
                self.send_response(response.status)
                for name, value in response.getheaders():
                    if name.lower() not in hop:
                        self.send_header(name, value)
                if body is not None:
                    self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                if body is not None:
                    self.wfile.write(body)
                else:
                    while chunk := response.read1(8192):
                        self.wfile.write(chunk)
                        self.wfile.flush()
            except (BrokenPipeError, ConnectionError, TimeoutError):
                pass
            finally:
                connection.close()

        do_GET = do_POST = do_PUT = do_DELETE = do_PATCH = forward

        def log_message(self, *_args):
            pass

    with ThreadingHTTPServer(("127.0.0.1", 0), Relay) as server:
        background = threading.Thread(target=server.serve_forever, daemon=True)
        background.start()
        try:
            yield f"http://127.0.0.1:{server.server_port}"
        finally:
            server.shutdown()
            background.join()


PROMPT = "Prepare the owned two-source intention."

def verify_appkit_send(binary, evidence, originals, root, backend, tool_result):
    rows = [json.loads(line) for line in (evidence/"appkit-http.jsonl").read_text().splitlines()]
    sends = [row for row in rows if row["method"] == "POST" and row["path"] == "/api/docs/workspace/agent"]
    send, = sends
    assert send["status"] == 202 and send["fixture_backend_translation"] == ["python", "owned-proposal"], send
    original = send["request"]
    assert original["prompt"] == PROMPT and original["parent_id"] is None, original
    buffers = original["context"]["buffers"]
    snapshots = {item["path"]: item for item in buffers}
    assert len(buffers) == 2 and set(snapshots) == {"note.py", "construction.md"}, buffers
    assert len({item["id"] for item in snapshots.values()}) == 2
    for path, item in snapshots.items():
        assert item["content"].encode() == originals[path] and item["save_guard"], item
        revisions = [row["source_revision"] for row in item["observations"] if row["kind"] == "editor_revision"]
        assert len(revisions) == 1 and revisions[0], item
    tool = json.loads(tool_result.read_text())["result"]
    assert not tool["isError"], tool
    token = json.loads(tool["content"][0]["text"])["token"]
    sealed = api(backend, "GET", "/api/agent-proposals/"+token)
    snapshot = json.loads(sealed["material"])["intention"]["editor_snapshot"]
    assert {row["id"] for row in snapshot["buffers"]} == {row["id"] for row in snapshots.values()}, snapshot
    assert sealed["host_application"]["saved"] is False and sealed["host_application"]["checks_executed"] is False
    assert all((root/name).read_bytes() == data for name,data in originals.items())
    assert not any(row["method"] in {"PUT", "PATCH", "DELETE"} or
             row["path"].startswith(("/api/changes/resource-save", "/api/source-history/"))
             for row in rows), rows
    assert any(row["method"] == "POST" and row["path"] == "/api/agent-proposals/validate" and row["status"] == 200 for row in rows)
    (evidence/"receipt.json").write_text(json.dumps({
        "engine": str(binary), "scope": "Mounted AppKit native Send; rendered review with native review/apply controller; workspace menu dispatch and native editor reflection",
        "native_send_request": original, "send_status": send["status"], "fixture_backend_translation": send["fixture_backend_translation"],
        "sealed": {"token": token, "fingerprint": sealed["fingerprint"]},
        "reported_draft_application": sealed["host_application"],
        "file_hashes_unchanged": {name: hashlib.sha256(data).hexdigest() for name, data in originals.items()},
        "log_sha256": hashlib.sha256((evidence/"appkit.log").read_bytes()).hexdigest(),
    }, indent=2)+"\n")

def appkit_send(fixture, evidence):
    binary = Path(os.environ["HICKORY_SIMPLE_TEST_ENGINE"]).resolve(strict=True)
    persistence = load_fixture("persistence", "test-agent-proposal-persistence.py")
    evidence.mkdir(parents=True)
    with tempfile.TemporaryDirectory(prefix="hickory-appkit-proposal-send-") as temporary:
        lease = Path(temporary).resolve()
        root, state = lease / "workspace", lease / "state"
        prepare(root)
        settings, calls, result = provider_fixture(lease)

        def edit(line, start, end):
            return {"range": {"start": {"line": line, "character": start},
                              "end": {"line": line, "character": end}}, "newText": "41"}

        changes = {"hick:///note.py": [edit(0, 6, 7)], "hick:///construction.md": [edit(2, 12, 13)]}
        calls.write_text(json.dumps({"intention": intention({"changes": changes})}))
        names = ("note.py", "construction.md", "generated.py", "project.md", "behavior.md")
        originals = {name: (root / name).read_bytes() for name in names}
        environment = {
            **os.environ, "HICKORY_STATE_DIR": str(state), "HICKORY_EXECUTOR": "local",
            "HICKORY_AGENT_TEST_UI_SETTINGS": str(settings), "HICKORY_APPKIT_PROPOSAL_SEND_ROOT": str(root),
            "HICKORY_APPKIT_PROPOSAL_SEND_IMAGES": str(evidence),
        }
        engine = fixture.start([str(binary), str(root), "appkit-proposal-send"], env=environment)
        try:
            origin = fixture.origin(engine)
            api(origin, "POST", "/api/docs/workspace/agent/acp", {"backend": "owned-proposal"})
            command = ["swift", "test", "--package-path", "apps/apple-simple",
                       "--scratch-path", "target/apple-simple-spm", "--jobs", "2",
                       "--filter", "AgentProposalTests/testMountedNativeAgentSendThroughCoordinatedDraftHistory"]
            with trace_appkit_send(origin, evidence / "appkit-http.jsonl") as relay, (evidence / "appkit.log").open("w") as output:
                environment["HICKORY_APPKIT_PROPOSAL_SEND_ORIGIN"] = relay
                test = subprocess.Popen(command, cwd=ROOT, env=environment, stdout=output,
                                        stderr=subprocess.STDOUT, start_new_session=True)
                try:
                    status = test.wait(timeout=100)
                except subprocess.TimeoutExpired:
                    os.killpg(test.pid, signal.SIGTERM)
                    test.wait(timeout=10)
                    raise
                if status:
                    raise subprocess.CalledProcessError(status, command)
            verify_appkit_send(binary, evidence, originals, root, origin, result)
        finally:
            persistence.stop_engine(fixture, engine, state)
            assert all((root / name).read_bytes() == saved for name, saved in originals.items())
    print(evidence / "receipt.json", flush=True)


def load_fixture(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
 parser = argparse.ArgumentParser()
 modes = parser.add_mutually_exclusive_group()
 modes.add_argument('--serve-only', action='store_true')
 modes.add_argument('--appkit-send-receipt', type=Path)
 args = parser.parse_args()
 fixture = load_fixture('browser_fixture', 'test-simple-browser.py')
 if args.appkit_send_receipt:
  if sys.platform != 'darwin' or not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
   parser.error('AppKit Send requires macOS and an explicit HICKORY_SIMPLE_TEST_ENGINE')
  appkit_send(fixture, args.appkit_send_receipt.resolve())
  return
 with tempfile.TemporaryDirectory(prefix='hickory-agent-proposals-browser-') as temporary:
  folder=Path(temporary).resolve();details=prepare(folder/'workspace');root=Path(details['workspace']);settings,calls,result=provider_fixture(folder);processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   environment={**os.environ,'HICKORY_STATE_DIR':str(folder/'state'),'HICKORY_AGENT_TEST_UI_SETTINGS':str(settings)}
   engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',f'http://127.0.0.1:{port}'],env=environment);processes.append(engine);origin=fixture.origin(engine)
   if args.serve_only:
    stopped=threading.Event()
    for event in (signal.SIGTERM,signal.SIGINT):signal.signal(event,lambda *_:stopped.set())
    print(json.dumps({'origin':origin,**details}),flush=True);stopped.wait();return
   vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**environment,'HICKORY_API_ORIGIN':origin});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path='note.py');driver.act('remember-editor',path='note.py');position=driver.act('editor-position',path='note.py',anchor=2);size=driver.act('editor-size',path='note.py');notes=driver.act('proposal-new-note',text='retained unrelated user notes');driver.act('remember-editor',path=notes['path']);driver.act('open',path='note.py');assert driver.act('editor-is-retained',path='note.py');captured=driver.act('proposal-editor-context',path='note.py')
   edit={'changes':{'hick:///note.py':[{'range':{'start':{'line':0,'character':6},'end':{'line':0,'character':7}},'newText':'41'}]}}
   summary=controlled_tool(origin,calls,result,captured,edit);assert (root/'note.py').read_text()==details['before']
   driver.act('proposal-open');text=driver.act('proposal-select',token=summary['token']);assert 'Agent claims' in text and 'unverified' in text and 'Proposed intention' in text,text[-4000:];assert 'engine-managed agent tool' in text
   state=driver.act('proposal-state');assert state['applyEnabled'],state
   driver.act('proposal-lose-report');applied=driver.act('proposal-apply');assert 'draft-applied' in applied and 'only this exact acknowledgment' in applied,applied[-4000:];assert driver.act('construction-source',path='note.py')==details['after'];assert (root/'note.py').read_text()==details['before'];assert driver.act('editor-is-retained',path='note.py');assert driver.act('editor-position',path='note.py')==position;assert driver.act('editor-size',path='note.py')==size
   assert api(origin,'GET','/api/agent-proposals/'+summary['token'])['host_application'] is None;driver.act('proposal-retry-report');reports=driver.act('proposal-report-bodies');assert len(reports)==2 and reports[0]==reports[1],reports;assert driver.act('construction-source',path='note.py')==details['after']
   retained=api(origin,'GET','/api/agent-proposals/'+summary['token']);assert retained['host_application']['status']=='host-reported draft application' and not retained['host_application']['saved'] and not retained['host_application']['checks_executed']
   driver.act('proposal-close');driver.act('workspace-history',redo=False);assert driver.act('construction-source',path='note.py')==details['before'];driver.act('workspace-history',redo=True);assert driver.act('construction-source',path='note.py')==details['after'];driver.act('construction-history',redo=False);assert driver.act('construction-source',path='note.py')==details['before'];driver.act('construction-history',redo=True);assert driver.act('construction-source',path='note.py')==details['after'];driver.act('workspace-history',redo=False);assert driver.act('construction-source',path='note.py')==details['before']
   captured=driver.act('proposal-editor-context',path='note.py');stale=controlled_tool(origin,calls,result,captured,edit);driver.act('proposal-open');driver.act('proposal-select',token=stale['token']);driver.act('type-source-selection',path='note.py',text='#stale');assert not driver.act('proposal-state')['applyEnabled'];assert (root/'note.py').read_text()==details['before'];driver.act('proposal-close');driver.act('construction-history',redo=False);assert driver.act('construction-source',path='note.py')==details['before']
   moved='moved λ%2F?#.py';move={'documentChanges':[{'kind':'rename','oldUri':'hick:///note.py','newUri':'hick:///'+urllib.parse.quote(moved,safe='')} ]};captured=driver.act('proposal-editor-context',path='note.py');resource=controlled_tool(origin,calls,result,captured,move);driver.act('proposal-open');text=driver.act('proposal-select',token=resource['token']);assert 'Save reviewed resource namespace' in text,text[-4000:];published=driver.act('proposal-apply',resources=True);assert 'resources-published' in published,published[-4000:];assert not (root/'note.py').exists() and (root/moved).read_text()==details['before'];assert driver.act('editor-is-retained',path=moved,previous='note.py');driver.act('proposal-close')
   driver.act('workspace-history',redo=False);driver.act('change-resource-ready');driver.act('change-resource-save');driver.act('change-close');assert (root/'note.py').read_text()==details['before'] and not (root/moved).exists();assert driver.act('editor-is-retained',path='note.py');driver.act('workspace-history',redo=True);driver.act('change-resource-ready');driver.act('change-resource-save');driver.act('change-close');assert (root/moved).read_text()==details['before'];assert (root/'generated.py').read_text()=='generated = 1\n';assert driver.act('editor-size',path=moved)==size
   # An independently added owner after Send cannot be blessed by taking a new review lease.
   captured=driver.act('proposal-editor-context',path=moved);later_edit={'changes':{'hick:///'+urllib.parse.quote(moved,safe=''):[{'range':{'start':{'line':0,'character':6},'end':{'line':0,'character':7}},'newText':'41'}]}};late=controlled_tool(origin,calls,result,captured,later_edit)
   late_path='late-construction.weird';late_text='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:include path="other.md"/></h:doc>';(root/late_path).write_text(late_text);driver.act('source-file-refresh');driver.act('open',path=late_path);driver.act('remember-editor',path=late_path);late_source=driver.act('construction-source',path=late_path,selection=True);driver.act('proposal-open');refused=driver.act('proposal-select',token=late['token']);assert not driver.act('proposal-state')['applyEnabled'];assert 'snapshot' in refused.lower() or 'not captured' in refused.lower(),refused[-3000:];assert driver.act('construction-source',path=late_path,selection=True)==late_source;assert driver.act('editor-is-retained',path=late_path);assert (root/moved).read_text()==details['before'];driver.act('proposal-close')
   driver.act('proposal-activate-note',**notes);assert driver.act('construction-source',path=notes['path'])=='retained unrelated user notes';assert driver.act('editor-is-retained',path=notes['path'])
   assert not driver.act('debug-diagnostics')['errors'];print('PASS controlled ACP tool→sealed whole intention→floating review→one coordinated draft Apply/Undo/Redo/native undo, unchanged saved bytes, stale editor refusal, separate guarded v2 namespace Save/adoption/resource Undo/Redo, retained editor/selection/geometry and unrelated unsaved note/cached editor, exact lost-report acknowledgment replay without reapplying, original editor snapshot identity/revision guards, host-only reports and no fabricated verification',flush=True)
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()

<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Exercise a controlled tool and the actual proposal host

For maintainers changing the boundary between an agent intention, current editor
ownership and saved resource publication. Every run owns its workspace, engine
state and ACP settings. The scripted provider invokes the real engine-managed
proposal tool; it performs no semantic judgment and uses no external account.
The fixture observes the real AgentSession's outgoing non-live snapshot once at
an explicit unavailable-provider boundary; it does not invent buffer IDs or edit
revisions. That exact snapshot is then supplied to the separately configured
scripted ACP connection. Its tool dispatch and retained engine-managed proposal
are real; the fixture503 is not provider sign-in or a semantic model response.

The browser then uses the existing menu and floating reader to review the whole
intention. Draft Apply, local/native Undo, resource Save, inverse restoration and
host acknowledgment remain separate outcomes.

The reusable `prepare(root)` requires an empty owned directory and writes the
native fixture marker. `--serve-only` exposes that isolated real engine for
native controller witnesses. A caller-provided API review is not upgraded to an
engine-managed tool attestation by this helper.

<ui:copy id="agent-proposals-browser-real-engine">
"""Owned engine/ACP/browser: sealed proposal review, whole draft Apply and resource Save."""
import argparse,hashlib,importlib.util,json,os,signal,socket,subprocess,sys,tempfile,threading,time,urllib.error,urllib.parse,urllib.request,uuid
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
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--serve-only',action='store_true');args=parser.parse_args()
 spec=importlib.util.spec_from_file_location('browser_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
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
</ui:copy>
<ui:file path="scripts/test-agent-proposals-browser.py"><ui:paste select="#agent-proposals-browser-real-engine"/></ui:file>

The maintained default witness passed against the real engine, local scripted
ACP process and headless browser. It verified whole-intention admission, original
opaque editor IDs and edit revisions, complete source/consequence/evidence
reading, precise draft transactions and both undo paths, an unrelated retained
unsaved note, separate resource publication and inverse restoration, stale and
unknown-owner refusal, unchanged generated bytes, and exact acknowledgment retry
without repeating Apply. The local three-method ACP script is a deliberate
protocol fixture: graceful provider unavailability alone cannot exercise a
positive tool invocation without credentials. Its limited supported methods
fail explicitly if the protocol changes. It makes no model judgment or test
verdict. The initial503 context capture is an explicit transport failure witness,
not successful provider sign-in. Physical input and accessibility acceptance
remain unexecuted.
</ui:doc>

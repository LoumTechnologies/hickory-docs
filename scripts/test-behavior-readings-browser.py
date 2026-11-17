"""Owned repository: connected readings, relevant verification and retained receipts."""
import argparse,hashlib,importlib.util,json,os,shutil,signal,socket,subprocess,tempfile,threading,time,urllib.error,urllib.request,urllib.parse,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def prepare(root):
 """Require an empty owned directory; never inspect checkout execution artifacts."""
 root.mkdir(parents=True,exist_ok=True);root=root.resolve()
 if any(root.iterdir()):raise ValueError('Behavior fixture requires an empty owned workspace')
 (root/'hickory-dogfood').mkdir();sibling=root/'sibling-ran.txt';manual=root/'manual-ran.txt';stale_manual=root/'stale-manual-ran.txt'
 implementation='def answer():\n    return 41\n';unrelated='def test_unrelated():\n    pass\n'
 test=f'from a import answer\nfrom pathlib import Path\ndef test_selected():\n    assert answer() == 41\ndef test_sibling():\n    Path({str(sibling)!r}).write_text("unwanted")\n'
 held_ready=root.parent/'held-ready.txt';held_release=root.parent/'held-release.txt'
 held_loop=f'while not Path({str(held_release)!r}).exists(): time.sleep(.05)'
 held_code=f'from pathlib import Path; import time; Path({str(held_ready)!r}).write_text("ready"); exec({held_loop!r}); print("owned process complete",flush=True)'
 launch={'version':1,'profiles':[{'id':'held-manual-check','name':'Held manual check','command':['python3','-c',held_code]},{'id':'manual-check','name':'Explicit manual check','command':['python3','-c',f'from pathlib import Path; Path({str(manual)!r}).write_text("ran")'],'default':True},{'id':'stale-manual-check','name':'Stale manual check','command':['python3','-c',f'from pathlib import Path; Path({str(stale_manual)!r}).write_text("unwanted")']}]}
 entry='<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:project sources="source.md"/>\n</h:doc>\n'
 marker='Later explanation for the declared answer'
 source=('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="a.py"><h:paste select="#answer"/></h:file>\n<h:file path="test_a.py">\n'+test+'</h:file>\n<h:file path="test_undeclared.py">\n'+unrelated+'</h:file>\n<h:file path=".hick-launch.json">\n'+json.dumps(launch)+'\n</h:file>\n<h:file path="pyproject.toml">\n[tool.uv]\n</h:file>\n'+('Earlier retained source context 😀.\n'*220)+'\n## '+marker+'\n\nOnly the explicit compiler and metadata relationships authorize this reading.\n<h:copy id="answer">\n'+implementation+'</h:copy>\n</h:doc>\n')
 files={'project.md':entry,'source.md':source,'a.py':implementation,'test_a.py':test,'test_undeclared.py':unrelated,'.hick-launch.json':json.dumps(launch)+'\n','pyproject.toml':'[tool.uv]\n','behavior.md':'# Answer behavior\nReturn the declared answer.\n','.hick-tests.json':json.dumps({'groups':{'selected':{'selectors':['test_a.py::test_selected'],'labels':['owned-behavior']}}})}
 for path,text in files.items():(root/path).write_text(text)
 (root/'.venv').symlink_to(ROOT/'target/thread-debug-env',target_is_directory=True)
 metadata={'format':1,'contracts':{'answer':{'page':'behavior.md','title':'Answer behavior','disposition':'retained-requirement','references':['a.py'],'tests':[{'runner':'pytest','file':'test_a.py','name':'test_selected'},{'runner':'unconfigured-check','file':'manual.check','name':'Explicit unsupported check'}]}},'constructors':{'a.py':{'source':'source.md'}}}
 narrative={'format':1,'paired_test_modules':[{'implementation':'a.py','tests':'test_a.py','page':'source.md'}],'fragments':[{'id':'answer','page':'source.md','output':'a.py','symbol':'answer','basis':'needs-explanation'}]}
 (root/'hickory-dogfood/behavior-map.json').write_text(json.dumps(metadata));(root/'hickory-dogfood/narrative-map.json').write_text(json.dumps(narrative));(root/'.hickory-native-behavior-fixture').write_text('owned-native-behavior-v1\n');subprocess.run(['git','init','-q',str(root)],check=True)
 return {'workspace':str(root),'entry':'project.md','focus':'a.py','source_file':'source.md','source_text':source,'implementation_text':implementation,'selected':'test_selected','unsupported':'Explicit unsupported check','profile':'Explicit manual check','sibling':str(sibling),'manual':str(manual),'stale_manual':str(stale_manual),'later_marker':marker,'fragment':'answer','held_ready':str(held_ready),'held_release':str(held_release)}
def api(origin,path,body=None,expected=200):
 request=urllib.request.Request(origin+path,data=None if body is None else json.dumps(body).encode(),headers={'Content-Type':'application/json'})
 try:
  with urllib.request.urlopen(request,timeout=30) as response:value=json.load(response)
  assert expected==200,(expected,value);return value
 except urllib.error.HTTPError as error:
  value=error.read().decode();assert error.code==expected,(error.code,value);return value

def verify_receipts(root):
 directory=root/'.hick-cache/behavior-verification';return set(directory.glob('*.json')) if directory.exists() else set()
def coverage_receipts(origin,root,source,implementation):
 # Real pytest-cov/JUnit collection, deliberately restricted to one selected case.
 record=api(origin,'/api/tests/coverage',{'action':'run','path':'.','language':'python','group':'selected'});deadline=time.monotonic()+20
 while time.monotonic()<deadline:
  value=api(origin,'/api/tests/coverage',{'action':'get','id':record['id']})
  if value['status'] not in ('running','collecting'):break
  time.sleep(.1)
 assert value['status']=='completed' and value['exit_code']==0,value
 assert any(test['status']=='passed' and 'test_selected' in test['id'] for test in value['tests']),value['tests']
 assert not (root/'sibling-ran.txt').exists()
 identity={'collector':'owned-manual-import','version':'1','configuration':'Explicit fixture import; not automated execution','scope':'.','source_hashes':{'source.md':hashlib.sha256(source.encode()).hexdigest(),'a.py':hashlib.sha256(implementation.encode()).hexdigest()}}
 raw='SF:a.py\nDA:1,1\nDA:2,1\nend_of_record\n'
 current=api(origin,'/api/tests/coverage',{'action':'import','path':'.','format':'lcov','raw':raw,'identity':identity})
 stale_identity={**identity,'source_hashes':{'a.py':'0'*64,'source.md':'0'*64}}
 api(origin,'/api/tests/coverage',{'action':'import','path':'.','format':'lcov','raw':raw,'identity':stale_identity})
 legacy=api(origin,'/api/tests/coverage',{'action':'import','path':'.','format':'lcov','raw':raw,'identity':identity})
 # Explicit retained legacy-input fixture, still labelled import: no fabricated automated record.
 path=root/'.hick-cache/coverage'/f"{legacy['id']}.json";data=json.loads(path.read_text());data.pop('workspace_id');path.write_text(json.dumps(data))
 return current['id']
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--serve-only',action='store_true');options=parser.parse_args()
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-behavior-reading-') as temporary:
  folder=Path(temporary);details=prepare(folder/'workspace');root=Path(details['workspace']);source=details['source_text'];implementation=details['implementation_text'];sibling=Path(details['sibling']);manual=Path(details['manual']);processes=[];driver=None
  environment={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')}
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   ui=f'http://127.0.0.1:{port}';engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
   if options.serve_only:
    stopped=threading.Event()
    for event in (signal.SIGTERM,signal.SIGINT):signal.signal(event,lambda *_:stopped.set())
    print(json.dumps({'origin':backend,**details}),flush=True);stopped.wait();return
   opened=api(backend,'/api/open-file',{'path':'source.md'});loaded=api(backend,'/api/docs/'+urllib.parse.quote(opened['doc_id'],safe=''))
   context={'version':1,'entry':'project.md','focus':'a.py','sources':{'source.md':source},'missing_sources':[],'unaddressed_sources':{},'saved_guards':{'source.md':loaded['save_guard']}};reading=api(backend,'/api/behavior/reading',context)
   # Revoke a captured profile before any editor is opened. Restored bytes do not
   # resurrect its old physical guard; the later editor receives its initial baseline explicitly.
   profile_scope={key:reading[key] for key in ['engine_workspace_id','workspace_id','reading_fingerprint']};profile_scope['context']=context
   profiles=api(backend,'/api/behavior/profiles',profile_scope);stale_choice=next(profile for profile in profiles['profiles'] if profile['id']=='stale-manual-check');stale_check=next(check for check in reading['graph']['checks'] if check['label']=='Explicit unsupported check')
   (root/'source.md').write_text(source.replace('Stale manual check','Changed stale profile'))
   api(backend,'/api/behavior/verify',{'version':1,**profile_scope,'check_id':stale_check['id'],'request_id':uuid.uuid4().hex,'profile':{'id':stale_choice['id'],'fingerprint':profiles['fingerprint']}},expected=409)
   (root/'source.md').write_text(source);assert not Path(details['stale_manual']).exists()
   loaded=api(backend,'/api/docs/'+urllib.parse.quote(opened['doc_id'],safe=''));context={**context,'saved_guards':{'source.md':loaded['save_guard']}};reading=api(backend,'/api/behavior/reading',context)
   assert reading['verification_available'] is True,reading['graph']['notices']
   # Readability survives missing loaded authority and anonymous construction ownership.
   for changed_context in [{**context,'saved_guards':{}},{**context,'unaddressed_sources':{'opaque-retained':'<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:file path="other.py">value=99</h:file></h:doc>'}}]:
    unavailable=api(backend,'/api/behavior/reading',changed_context);assert unavailable['verification_available'] is False and any(node['label']=='Answer behavior' for node in unavailable['graph']['nodes'].values()),unavailable
    selected_check=next(check for check in unavailable['graph']['checks'] if check['available']);api(backend,'/api/behavior/verify',{'version':1,'engine_workspace_id':unavailable['engine_workspace_id'],'workspace_id':unavailable['workspace_id'],'reading_fingerprint':unavailable['reading_fingerprint'],'check_id':selected_check['id'],'request_id':uuid.uuid4().hex,'context':changed_context},expected=409)
   unrelated=root/'retained-notes.txt';unrelated.write_text('ordinary retained notes');note=api(backend,'/api/file?path=retained-notes.txt');guarded_context={**context,'sources':{**context['sources'],'retained-notes.txt':'ordinary retained notes'},'saved_guards':{**context['saved_guards'],'retained-notes.txt':note['save_guard']}}
   assert api(backend,'/api/behavior/reading',guarded_context)['verification_available'] is True
   dirty_named={**guarded_context,'sources':{**guarded_context['sources'],'retained-notes.txt':'unsaved retained notes'}};assert api(backend,'/api/behavior/reading',dirty_named)['verification_available'] is False
   replacement=root/'replacement-notes.txt';replacement.write_text(unrelated.read_text());replacement.replace(unrelated)
   stale_guard=api(backend,'/api/behavior/reading',guarded_context);assert stale_guard['verification_available'] is False and stale_guard['graph']['checks'],stale_guard
   output=next(output for output in reading['construction']['outputs'] if output['path']=='a.py');assert output['after']['hash']==hashlib.sha256(implementation.encode()).hexdigest(),output
   vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**environment,'HICKORY_API_ORIGIN':backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path='source.md');driver.act('remember-editor',path='source.md');size=driver.act('editor-size',path='source.md');before=driver.act('construction-source',path='source.md',selection=True)
   driver.act('construction-open');driver.act('construction-read',entry='project.md',focus='a.py');text=driver.act('behavior-ready');assert 'Answer behavior' in text and 'needs-explanation' in text,text[-3000:]
   assert not any('test_unrelated' in option for option in driver.act('behavior-check-options'))
   page=driver.act('behavior-source',kind='explanation',label='answer');assert details['later_marker'] in page['text'] and 'observed-fragment-context' in page['state'],page
   assert driver.act('construction-source',path='source.md',selection=True)==before
   driver.act('construction-close');driver.act('open',path='retained-notes.txt');driver.act('remember-editor',path='retained-notes.txt');driver.act('editor-position',path='retained-notes.txt',anchor=0);driver.act('type-source-selection',path='retained-notes.txt',text='unsaved ');dirty_notes=driver.act('construction-source',path='retained-notes.txt',selection=True)
   driver.act('construction-open');driver.act('construction-read',entry='project.md',focus='a.py');unavailable=driver.act('behavior-ready');assert 'unsaved' in unavailable.lower() and not driver.act('behavior-run-enabled'),unavailable[-3000:];assert driver.act('construction-source',path='retained-notes.txt',selection=True)==dirty_notes;assert driver.act('editor-is-retained',path='retained-notes.txt');assert unrelated.read_text()=='ordinary retained notes'
   driver.act('construction-close');driver.act('construction-history',redo=False);driver.act('open',path='source.md');assert driver.act('editor-is-retained',path='source.md');assert driver.act('construction-source',path='source.md',selection=True)==before;driver.act('construction-open');driver.act('construction-read',entry='project.md',focus='a.py');driver.act('behavior-ready')
   surface=driver.act('behavior-surface');assert surface['position']=='fixed' and surface['width']<=450 and surface['bottom']<=surface['viewport'],surface
   driver.act('behavior-check',label='test_selected');driver.act('behavior-run');text=driver.act('behavior-finished');assert 'process-finished' in text and 'process-only' in text,text[-3000:];assert not sibling.exists()
   driver.act('behavior-check',label='Explicit unsupported check');driver.act('behavior-profiles',label='Explicit manual check');driver.act('behavior-profile-run');text=driver.act('behavior-finished');assert 'user-declared profile' in text,text[-3000:];assert manual.read_text()=='ran'
   # Same request ID can inspect only the exact typed intent, including selected profile.
   body=driver.act('behavior-last-request');api(backend,'/api/behavior/verify',body);changed={**body,'profile':{**body['profile'],'id':'stale-manual-check'}};api(backend,'/api/behavior/verify',changed,expected=409)
   selected=next(check for check in reading['graph']['checks'] if check['label']=='test_selected' and check['available']);proof={'version':1,'engine_workspace_id':reading['engine_workspace_id'],'workspace_id':reading['workspace_id'],'reading_fingerprint':reading['reading_fingerprint'],'check_id':selected['id'],'context':context}
   existing=verify_receipts(root);api(backend,'/api/behavior/verify',{**proof,'request_id':uuid.uuid4().hex,'workspace_id':'f'*64},expected=409);assert verify_receipts(root)==existing
   (root/'a.py').write_text(implementation.replace('41','99'));api(backend,'/api/behavior/verify',{**proof,'request_id':uuid.uuid4().hex},expected=409);assert verify_receipts(root)==existing;(root/'a.py').write_text(implementation)
   assert verify_receipts(root)==existing
   coverage_receipts(backend,root,source,implementation)
   captured=api(backend,'/api/behavior/reading',context);rows=captured['evidence']['records'];assert any(row['evidence_kind']=='automated-collector' and row['freshness']=='current-captured-bytes' for row in rows),rows;assert any(row['freshness']=='stale' for row in rows),rows;assert any(row['origin']=='unknown' for row in rows),rows;assert any(row['evidence_kind']=='manual-or-imported' for row in rows),rows
   driver.act('construction-read',entry='project.md',focus='a.py');driver.act('behavior-ready');evidence=driver.act('behavior-evidence');assert 'automated-collector' in evidence and 'manual-or-imported' in evidence and 'origin-unknown' in evidence and 'stale' in evidence,evidence[-4000:]
   # Capability reading preserves a live unsaved draft and cannot run a superseded review.
   driver.act('source-tests-edit',path='source.md',text='\n# retained unsaved review draft\n');dirty=driver.act('construction-source',path='source.md',selection=True);count=driver.act('behavior-request-count');driver.act('behavior-check',label='test_selected');assert not driver.act('behavior-run-enabled');assert driver.act('behavior-request-count')==count
   driver.act('behavior-source',kind='explanation',label='answer');assert driver.act('construction-source',path='source.md',selection=True)==dirty
   driver.act('construction-close');driver.act('construction-open');assert 'process-finished' in driver.act('behavior-ready',retained=True);driver.act('construction-close')
   assert driver.act('editor-is-retained',path='source.md');assert driver.act('editor-size',path='source.md')==size
   driver.act('construction-history',redo=False);assert driver.act('construction-source',path='source.md')==source;driver.act('construction-history',redo=True);assert driver.act('construction-source',path='source.md')==dirty['source']
   assert (root/'source.md').read_text()==source and (root/'a.py').read_text()==implementation;assert not sibling.exists();assert not driver.act('debug-diagnostics')['errors']
   # The process belongs to the admitted engine, independently of its source directory.
   driver.act('construction-history',redo=False);driver.act('construction-open');driver.act('construction-read',entry='project.md',focus='a.py');driver.act('behavior-ready');driver.act('behavior-check',label='Explicit unsupported check');driver.act('behavior-profiles',label='Held manual check');driver.act('behavior-profile-run')
   deadline=time.monotonic()+10
   while time.monotonic()<deadline and not Path(details['held_ready']).exists():time.sleep(.05)
   assert Path(details['held_ready']).read_text()=='ready'
   held=driver.act('behavior-last-request');owned=api(backend,'/api/behavior/verification?id='+held['request_id']);assert owned['terminal']['id'];known_terminal=owned['terminal']['id']
   # Identical bytes cannot rebind an old reading to a replacement directory.
   retired=root.with_name('retained-original');root.rename(retired);shutil.copytree(retired,root,symlinks=True)
   cache_before={path.name:path.read_bytes() for path in verify_receipts(root)}
   observation=api(backend,'/api/behavior/verification?id='+held['request_id']);assert observation['workspace_current'] is False and observation['terminal']['id']==known_terminal,observation
   replay=api(backend,'/api/behavior/verify',held);assert replay['terminal']['id']==known_terminal and replay['workspace_current'] is False
   Path(details['held_release']).write_text('release');text=driver.act('behavior-finished');assert 'process-finished' in text and 'Workspace physical identity changed' in text,text[-3000:]
   final=api(backend,'/api/behavior/verification?id='+held['request_id']);assert final['terminal']['id']==known_terminal and final['status']=='process-finished' and final['exit_code']==0,final
   assert {path.name:path.read_bytes() for path in verify_receipts(root)}==cache_before
   assert driver.act('editor-is-retained',path='source.md') and driver.act('editor-size',path='source.md')==size
   identity=uuid.uuid4().hex;api(backend,'/api/behavior/verify',{**proof,'request_id':identity},expected=409);assert not (root/'.hick-cache/behavior-verification'/f'{identity}.json').exists()
   fixture.stop(engine)
   restarted=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',ui],env={**environment,'HICKORY_STATE_DIR':str(folder/'restarted-state')});processes.append(restarted);new_origin=fixture.origin(restarted)
   fresh=api(new_origin,'/api/behavior/reading',context);assert fresh['workspace_id']!=reading['workspace_id'];assert fresh['engine_workspace_id']!=reading['engine_workspace_id']
   api(new_origin,'/api/behavior/verify',{**proof,'request_id':identity},expected=409)
   api(new_origin,'/api/behavior/verify',{**proof,'engine_workspace_id':fresh['engine_workspace_id'],'request_id':identity},expected=409)
   assert not (root/'.hick-cache/behavior-verification'/f'{identity}.json').exists()
   print('PASS: later-fragment full captured reading; explicit contract/explanation/compiler/test edges; unrelated-test exclusion; selected pytest and exact user profile; full-intent replay/stale output/inventory/foreign refusal; genuine pytest-cov/JUnit plus manual/stale/legacy distinctions; retained process-only links, editor/selection/geometry/unsaved/native undo/redo, unchanged source bytes; identical-byte root replacement, independently owned post-effect process retention without foreign cache writes, and old-reading/new-engine refusal')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()

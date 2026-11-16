"""Owned repository: connected readings, relevant verification and retained receipts."""
import argparse,hashlib,importlib.util,json,os,shutil,signal,socket,subprocess,tempfile,threading,time,urllib.error,urllib.request,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def prepare(root):
 """Require an empty owned directory; never inspect checkout execution artifacts."""
 root.mkdir(parents=True,exist_ok=True);root=root.resolve()
 if any(root.iterdir()):raise ValueError('Behavior fixture requires an empty owned workspace')
 (root/'hickory-dogfood').mkdir();sibling=root/'sibling-ran.txt';manual=root/'manual-ran.txt';stale_manual=root/'stale-manual-ran.txt'
 implementation='def answer():\n    return 41\n';unrelated='def test_unrelated():\n    pass\n'
 test=f'from a import answer\nfrom pathlib import Path\ndef test_selected():\n    assert answer() == 41\ndef test_sibling():\n    Path({str(sibling)!r}).write_text("unwanted")\n'
 launch={'version':1,'profiles':[{'id':'manual-check','name':'Explicit manual check','command':['python3','-c',f'from pathlib import Path; Path({str(manual)!r}).write_text("ran")'],'default':True},{'id':'stale-manual-check','name':'Stale manual check','command':['python3','-c',f'from pathlib import Path; Path({str(stale_manual)!r}).write_text("unwanted")']}]}
 entry='<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:project sources="source.md"/>\n</h:doc>\n'
 marker='Later explanation for the declared answer'
 source=('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="a.py"><h:paste select="#answer"/></h:file>\n<h:file path="test_a.py">\n'+test+'</h:file>\n<h:file path="test_undeclared.py">\n'+unrelated+'</h:file>\n<h:file path=".hick-launch.json">\n'+json.dumps(launch)+'\n</h:file>\n<h:file path="pyproject.toml">\n[tool.uv]\n</h:file>\n'+('Earlier retained source context 😀.\n'*220)+'\n## '+marker+'\n\nOnly the explicit compiler and metadata relationships authorize this reading.\n<h:copy id="answer">\n'+implementation+'</h:copy>\n</h:doc>\n')
 files={'project.md':entry,'source.md':source,'a.py':implementation,'test_a.py':test,'test_undeclared.py':unrelated,'.hick-launch.json':json.dumps(launch)+'\n','pyproject.toml':'[tool.uv]\n','behavior.md':'# Answer behavior\nReturn the declared answer.\n','.hick-tests.json':json.dumps({'groups':{'selected':{'selectors':['test_a.py::test_selected'],'labels':['owned-behavior']}}})}
 for path,text in files.items():(root/path).write_text(text)
 (root/'.venv').symlink_to(ROOT/'target/thread-debug-env',target_is_directory=True)
 metadata={'format':1,'contracts':{'answer':{'page':'behavior.md','title':'Answer behavior','disposition':'retained-requirement','references':['a.py'],'tests':[{'runner':'pytest','file':'test_a.py','name':'test_selected'},{'runner':'unconfigured-check','file':'manual.check','name':'Explicit unsupported check'}]}},'constructors':{'a.py':{'source':'source.md'}}}
 narrative={'format':1,'paired_test_modules':[{'implementation':'a.py','tests':'test_a.py','page':'source.md'}],'fragments':[{'id':'answer','page':'source.md','output':'a.py','symbol':'answer','basis':'needs-explanation'}]}
 (root/'hickory-dogfood/behavior-map.json').write_text(json.dumps(metadata));(root/'hickory-dogfood/narrative-map.json').write_text(json.dumps(narrative));(root/'.hickory-native-behavior-fixture').write_text('owned-native-behavior-v1\n');subprocess.run(['git','init','-q',str(root)],check=True)
 return {'workspace':str(root),'entry':'project.md','focus':'a.py','source_file':'source.md','source_text':source,'implementation_text':implementation,'selected':'test_selected','unsupported':'Explicit unsupported check','profile':'Explicit manual check','sibling':str(sibling),'manual':str(manual),'stale_manual':str(stale_manual),'later_marker':marker,'fragment':'answer'}
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
   context={'version':1,'entry':'project.md','focus':'a.py','sources':{'source.md':source},'missing_sources':[]};reading=api(backend,'/api/behavior/reading',context)
   output=next(output for output in reading['construction']['outputs'] if output['path']=='a.py');assert output['after']['hash']==hashlib.sha256(implementation.encode()).hexdigest(),output
   vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**environment,'HICKORY_API_ORIGIN':backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path='source.md');driver.act('remember-editor',path='source.md');size=driver.act('editor-size',path='source.md');before=driver.act('construction-source',path='source.md',selection=True)
   driver.act('construction-open');driver.act('construction-read',entry='project.md',focus='a.py');text=driver.act('behavior-ready');assert 'Answer behavior' in text and 'needs-explanation' in text,text[-3000:]
   assert not any('test_unrelated' in option for option in driver.act('behavior-check-options'))
   page=driver.act('behavior-source',kind='explanation',label='answer');assert details['later_marker'] in page['text'] and 'observed-fragment-context' in page['state'],page
   assert driver.act('construction-source',path='source.md',selection=True)==before
   surface=driver.act('behavior-surface');assert surface['position']=='fixed' and surface['width']<=450 and surface['bottom']<=surface['viewport'],surface
   driver.act('behavior-check',label='test_selected');driver.act('behavior-run');text=driver.act('behavior-finished');assert 'process-finished' in text and 'process-only' in text,text[-3000:];assert not sibling.exists()
   driver.act('behavior-check',label='Explicit unsupported check');driver.act('behavior-profiles',label='Explicit manual check');driver.act('behavior-profile-run');text=driver.act('behavior-finished');assert 'user-declared profile' in text,text[-3000:];assert manual.read_text()=='ran'
   # Same request ID can inspect only the exact typed intent, including selected profile.
   body=driver.act('behavior-last-request');api(backend,'/api/behavior/verify',body);changed={**body,'profile':{**body['profile'],'id':'stale-manual-check'}};api(backend,'/api/behavior/verify',changed,expected=409)
   selected=next(check for check in reading['graph']['checks'] if check['label']=='test_selected' and check['available']);proof={'version':1,'engine_workspace_id':reading['engine_workspace_id'],'workspace_id':reading['workspace_id'],'reading_fingerprint':reading['reading_fingerprint'],'check_id':selected['id'],'context':context}
   existing=verify_receipts(root);api(backend,'/api/behavior/verify',{**proof,'request_id':uuid.uuid4().hex,'workspace_id':'f'*64},expected=409);assert verify_receipts(root)==existing
   (root/'a.py').write_text(implementation.replace('41','99'));api(backend,'/api/behavior/verify',{**proof,'request_id':uuid.uuid4().hex},expected=409);assert verify_receipts(root)==existing;(root/'a.py').write_text(implementation)
   # Explicit profile inventory revocation refuses before process creation.
   driver.act('behavior-profiles',label='Stale manual check');(root/'source.md').write_text(source.replace('Stale manual check','Changed stale profile'));driver.act('behavior-profile-run');refused=driver.act('behavior-refused');assert 'changed' in refused.lower(),refused;(root/'source.md').write_text(source);assert not Path(details['stale_manual']).exists()
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
   # Identical bytes cannot rebind an old reading to a replacement directory.
   retired=root.with_name('retained-original');root.rename(retired);shutil.copytree(retired,root,symlinks=True)
   identity=uuid.uuid4().hex;api(backend,'/api/behavior/verify',{**proof,'request_id':identity},expected=409);assert not (root/'.hick-cache/behavior-verification'/f'{identity}.json').exists()
   fixture.stop(engine)
   restarted=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',ui],env=environment);processes.append(restarted);new_origin=fixture.origin(restarted)
   fresh=api(new_origin,'/api/behavior/reading',context);assert fresh['workspace_id']!=reading['workspace_id'];assert fresh['engine_workspace_id']!=reading['engine_workspace_id']
   api(new_origin,'/api/behavior/verify',{**proof,'request_id':identity},expected=409)
   api(new_origin,'/api/behavior/verify',{**proof,'engine_workspace_id':fresh['engine_workspace_id'],'request_id':identity},expected=409)
   assert not (root/'.hick-cache/behavior-verification'/f'{identity}.json').exists()
   print('PASS: later-fragment full captured reading; explicit contract/explanation/compiler/test edges; unrelated-test exclusion; selected pytest and exact user profile; full-intent replay/stale output/inventory/foreign refusal; genuine pytest-cov/JUnit plus manual/stale/legacy distinctions; retained process-only links, editor/selection/geometry/unsaved/native undo/redo, unchanged source bytes; identical-byte root replacement and old-reading/new-engine refusal')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()

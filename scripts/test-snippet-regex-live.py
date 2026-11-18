"""Owned real-engine v3 completion/template host flows; no canned engine responses."""
import argparse,importlib.util,json,os,socket,subprocess,sys,tempfile,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def module(path,name):
 spec=importlib.util.spec_from_file_location(name,path);value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--host',choices=['protocol','browser','apple','android'],required=True);parser.add_argument('--template',action='store_true');args=parser.parse_args();fixture=module(ROOT/'scripts/test-simple-browser.py','simple_fixture');native=module(ROOT/'scripts/test-native-templates.py','native_fixture');cases=json.loads((ROOT/'apps/simple-contract/snippet-regex-live-cases.json').read_text())
 with tempfile.TemporaryDirectory(prefix='snippet-regex-v3-',dir=ROOT/'target') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.git').mkdir();(root/'.hickory').mkdir();(root/'.hickory-snippet-regex-fixture').write_text('owned-snippet-regex-v3\n');events=folder/'provider-events.jsonl'
  for path in ['regex.py','inline.py','budget.py']:(root/path).write_bytes(cases['source'].encode())
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':[sys.executable,str(ROOT/'scripts/snippet-regex-controlled-provider.py'),str(events)]}}}))
  (root/'.hickory/templates.json').write_text(json.dumps({'version':1,'templates':[{'id':'regex.mirrors','label':'Regex mirrors','description':'Lookbehind, escaped names, references, v sets, modifiers and emoji strings','language':'python','syntax':cases['syntax']},{'id':'regex.budget','label':'Budget refusal','description':'A failed calculation must refuse the entire field edit','language':'python','syntax':cases['budget_syntax']}]}))
  processes=[];driver=None
  try:
   with socket.socket() as reserve:reserve.bind(('127.0.0.1',0));port=reserve.getsockname()[1]
   ui=f'http://127.0.0.1:{port}';env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',ui],env=env);processes.append(engine);origin=fixture.origin(engine)
   if args.host=='protocol':subprocess.run(['node',str(ROOT/'scripts/snippet-regex-protocol-driver.mjs'),origin,str(events),str(root)],check=True,timeout=120)
   elif args.host=='browser':
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html')
    for path in ['regex.py','budget.py']:
     driver.act('open',path=path);driver.act('remember-editor',path=path);size=driver.act('editor-size',path=path)
     if args.template:
      driver.act('editor-position',path=path,anchor=0,head=6);catalog=driver.act('templates-open');label='Regex mirrors' if path=='regex.py' else 'Budget refusal';assert label in catalog,catalog;driver.act('templates-preview',label=label);assert driver.act('construction-source',path=path)==cases['source'];driver.act('templates-apply',path=path)
     else:
      deadline=time.monotonic()+20
      while True:
       options=driver.act('completion-open',path=path,offset=0,selection={'anchor':0,'head':6})
       if any('Regex mirrors' in option for option in options):break
       assert time.monotonic()<deadline,options;time.sleep(.1)
      notice=driver.act('completion-choose',title='Regex mirrors');assert 'Completion applied' in notice,notice
     expected=cases['after'] if path=='regex.py' else cases['budget_after'];assert driver.act('construction-source',path=path)==expected;assert driver.act('selected-source-text',path=path)=='OldOld'
     if path=='regex.py':
      driver.act('type-source-selection',path=path,text='NewNew');assert driver.act('construction-source',path=path)==cases['typed'];driver.act('undo',path=path);assert driver.act('construction-source',path=path)==expected;driver.act('redo',path=path);assert driver.act('construction-source',path=path)==cases['typed'];driver.act('undo',path=path)
      driver.act('type-source-selection',path=path,text='😀');assert driver.act('construction-source',path=path)==cases['emoji'];driver.act('undo',path=path);assert driver.act('construction-source',path=path)==expected
     else:
      driver.act('type-source-selection',path=path,text='a'*100+'!');assert driver.act('construction-source',path=path)==expected,'Refused VM calculation changed a primary field';assert driver.act('selected-source-text',path=path)=='OldOld'
     driver.act('undo',path=path);assert driver.act('construction-source',path=path)==cases['source'];driver.act('redo',path=path);assert driver.act('construction-source',path=path)==expected
     assert driver.act('editor-is-retained',path=path);assert driver.act('editor-size',path=path)==size
     if not args.template:
      observed=[json.loads(row) for row in events.read_text().splitlines()];assert any(row.get('initialize_snippets') is True for row in observed),observed;assert sum(row.get('resolve')==(root/path).resolve().as_uri() for row in observed)==1,observed
    print('PASS: rendered v3 '+('template' if args.template else 'completion')+' expansion, linked Unicode input, whole-edit budget refusal, atomic UndoRedo and retained code space/identity')
   else:
    if args.template:case=('SnippetRegexTests','testLiveRegexTemplatesAndHistory','regex-template') if args.host=='apple' else ('SnippetRegexTest','liveRegexTemplatesAndHistory','regex-template')
    else:case=('SnippetRegexTests','testLiveRegexProviderAndHistory','regex-provider') if args.host=='apple' else ('SnippetRegexTest','liveRegexProviderAndHistory','regex-provider')
    native.run_host(args.host,origin,root,folder,fixture,acceptance=case)
   for path in ['regex.py','inline.py','budget.py']:assert (root/path).read_bytes()==cases['source'].encode(),'V3 source preview/typing wrote disk bytes'
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()

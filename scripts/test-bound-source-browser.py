#!/usr/bin/env python3
"""Owned real-engine editor formatting and cross-source rename acceptance."""
import importlib.util,json,os,socket,sys,tempfile,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-bound-browser-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();events=folder/'events.jsonl';gate=folder/'gate'
  literate='--literate' in sys.argv
  if literate:
   spec=importlib.util.spec_from_file_location('literate_fixture',ROOT/'scripts/test-provider-bulk-literate.py');data=importlib.util.module_from_spec(spec);spec.loader.exec_module(data);sources=data.SOURCE;path='project.md';source=sources[path]
   for name,text in sources.items():(root/name).write_text(text)
  else:
   path='literal%2F λ?#.py';source='value = 1\n';hidden='value = 2\n';(root/path).write_text(source);(root/'hidden.py').write_text(hidden)
  provider='controlled-bulk-literate-provider.py' if literate else 'controlled-bound-source-provider.py'
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':[sys.executable,str(ROOT/'scripts'/provider),str(events),str(gate)]}}}))
  env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',f'http://127.0.0.1:{port}'],env=env);processes.append(engine);backend=fixture.origin(engine)
   vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path=path);driver.act('remember-editor',path=path);size=driver.act('editor-size',path=path)
   def prepare(command,name=None,choice=None):
    deadline=time.monotonic()+20
    while True:
     result=driver.act('change-prepare',command=command,**({'name':name} if name else {}))
     if (choice in result if choice else 'Apply to source drafts' in result):return result
     if 'Choose writable source with a connected language provider' not in result or time.monotonic()>deadline:raise AssertionError(result)
     time.sleep(.05)
   def inverse(direction):
    driver.act('change-open',command=direction+' workspace change');driver.act('change-close')
   if literate:
    dirty=sources['a.md'].replace('😀','draft 😀');driver.act('open',path='a.md');driver.act('construction-set-source',path='a.md',source=dirty);driver.act('open',path=path);time.sleep(.4)
    title='Fix complete file · out/a.py';driver.act('change-open',command='Preview file-wide diagnostic fixes');prepare('Prepare file-wide diagnostic fixes',choice=title);driver.act('action-choose',title=title);assert driver.act('construction-source',path=path)==source
    driver.act('change-apply');driver.act('change-close');driver.act('open',path='a.md');assert driver.act('construction-source',path='a.md')==dirty.replace('value','fixed');inverse('Undo');assert driver.act('construction-source',path='a.md')==dirty;inverse('Redo');assert driver.act('construction-source',path='a.md')==dirty.replace('value','fixed');inverse('Undo')
    driver.act('open',path=path);assert driver.act('editor-is-retained',path=path) and driver.act('editor-size',path=path)==size
    for name,text in sources.items():assert (root/name).read_text()==text,name
    assert not (root/'out').exists();records=[json.loads(line) for line in events.read_text().splitlines()];assert any(row.get('query','').endswith('/a.py') and 'draft 😀' in row.get('source','') for row in records);assert not driver.act('debug-diagnostics')['errors'];print('PASS rendered literate whole-file fix with dirty contributor, exact graph proof, fresh acquisition, draft Apply/Undo/Redo, retained root editor/geometry and unchanged Markdown/output absence');return
   driver.act('change-open',command='Preview formatting');prepare('Prepare formatting');assert driver.act('construction-source',path=path)==source
   driver.act('change-apply');driver.act('change-close');assert driver.act('construction-source',path=path)=='formatted = 1\n';assert (root/path).read_text()==source
   inverse('Undo');assert driver.act('construction-source',path=path)==source;inverse('Redo');assert driver.act('construction-source',path=path)=='formatted = 1\n';inverse('Undo')
   assert driver.act('editor-is-retained',path=path) and driver.act('editor-size',path=path)==size
   assert not any('hidden.py' in tab for tab in driver.act('tabs'))
   driver.act('change-open',command='Rename symbol');prepare('Prepare rename','discover');assert not any('hidden.py' in tab for tab in driver.act('tabs'))
   driver.act('change-apply');driver.act('change-close');driver.act('open',path='hidden.py');assert driver.act('construction-source',path='hidden.py')=='discover = 2\n';inverse('Undo');assert driver.act('construction-source',path='hidden.py')==hidden
   driver.act('open',path=path);assert driver.act('editor-is-retained',path=path) and driver.act('editor-size',path=path)==size;assert driver.act('construction-source',path=path)==source
   assert (root/path).read_text()==source and (root/'hidden.py').read_text()==hidden
   records=[json.loads(line) for line in events.read_text().splitlines()];assert len([row for row in records if row.get('method')=='textDocument/formatting'])>=2;assert len([row for row in records if row.get('name')=='discover'])>=2
   assert not driver.act('debug-diagnostics')['errors'];print('PASS bound editor formatting and unopened-source rename, fresh queries, complete preview, draft Apply/Undo/Redo, literal source identity, original editor/geometry and unchanged saved bytes')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()

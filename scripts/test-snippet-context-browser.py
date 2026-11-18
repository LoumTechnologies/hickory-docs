"""Actual blank-prefix popup, variable resolution, linked transforms and source history."""
import importlib.util,json,os,re,socket,sys,tempfile,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('simple_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
def main():
 with tempfile.TemporaryDirectory(prefix='snippet-context-browser-',dir=ROOT/'target') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();template='--template' in sys.argv;selected='--selection' in sys.argv or template;path='browser.py';source='old\r\nsecond\n';(root/path).write_bytes(source.encode());events=folder/'events.jsonl'
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':[sys.executable,str(ROOT/'scripts/snippet-context-controlled-provider.py'),str(events)]}}}))
  if template:
   (root/'.hickory').mkdir();(root/'.hickory/templates.json').write_text(json.dumps({'version':1,'templates':[{'id':'context.derived','label':'Context derived','description':'Captured source variables and computed mirrors','language':'python','syntax':'${TM_FILENAME}|${CURRENT_YEAR}|${RANDOM_HEX}|${1:${TM_SELECTED_TEXT}}|$1|${1/(.*)/${1:/upcase}/}$0'}]}))
  processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};ui=f'http://127.0.0.1:{port}'
   engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',ui],env=env);processes.append(engine);backend=fixture.origin(engine)
   vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path=path);driver.act('remember-editor',path=path);size=driver.act('editor-size',path=path);deadline=time.monotonic()+20
   if template:
    driver.act('editor-position',path=path,anchor=0,head=3);catalog=driver.act('templates-open');assert 'Context derived' in catalog,catalog;preview=driver.act('templates-preview',label='Context derived');assert 'browser.py|' in preview,preview;assert driver.act('construction-source',path=path)==source;driver.act('templates-apply',path=path)
   else:
    while True:
     options=driver.act('completion-open',path=path,offset=0,**({'selection':{'anchor':0,'head':3}} if selected else {}))
     if any('context completion' in option for option in options):break
     assert time.monotonic()<deadline,options;time.sleep(.1)
    notice=driver.act('completion-choose',title='context completion');assert 'Completion applied' in notice,notice
   after=driver.act('construction-source',path=path);pattern=r'browser\.py\|\d{4}\|[a-f0-9]{6}\|'+('old\|old\|OLD' if selected else '\|\|')+r'\r\nsecond\n';assert re.fullmatch(pattern,after),after
   assert driver.act('selected-source-text',path=path)==('old' if selected else '');driver.act('type-source-selection',path=path,text='ßa');typed=after.replace('old|old|OLD\r\n' if selected else '|||\r\n','ßa|ßa|SSA\r\n' if selected else '|ßa|ßa|SSA\r\n');assert driver.act('construction-source',path=path)==typed
   driver.act('undo',path=path);assert driver.act('construction-source',path=path)==after;driver.act('undo',path=path);assert driver.act('construction-source',path=path)==source
   driver.act('redo',path=path);assert driver.act('construction-source',path=path)==after;driver.act('redo',path=path);assert driver.act('construction-source',path=path)==typed
   assert driver.act('editor-is-retained',path=path);assert driver.act('editor-size',path=path)==size;assert (root/path).read_bytes()==source.encode()
   if not template:
    observed=[json.loads(row) for row in events.read_text().splitlines()];assert len([row for row in observed if row.get('resolve')])==1,observed
   print('PASS: actual '+('template preview/Apply' if template else 'explicit completion')+', captured source variables and selection, computed Unicode mirrors, atomic UndoRedo, retained editor geometry and unchanged CRLF disk bytes')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()

"""Drive embedded templates through the real source editor and shared compiler."""
import importlib.util,json,os,socket,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('simple_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
NS='http://www.hickorydocs.com/1.0'
def doc(body):return '<h:doc xmlns:h="'+NS+'">'+body+'</h:doc>'
def project(target='saved.py',mixed=False):
 body='<h:project sources="part.md"/><h:file path="'+target+'"><h:paste select="#body"/></h:file><h:file path="second.py"><h:paste select="#body"/></h:file>'
 if mixed:body+='<h:file path="mixed.js"><h:paste select="#body"/></h:file>'
 return doc(body)
def exercise(driver,root,path,source):
 driver.act('open',path=path);driver.act('remember-editor',path=path);size=driver.act('editor-size',path=path);position=source.index('old');driver.act('editor-position',path=path,anchor=position,head=position+3)
 catalog=driver.act('templates-open',path=path);assert 'Embedded context' in catalog,catalog
 preview=driver.act('templates-preview',label='Embedded context');expanded='#'+path+'|old|old|OLD';assert expanded in preview,preview;assert driver.act('construction-source',path=path)==source
 driver.act('templates-apply',path=path);after=source[:position]+expanded+source[position+3:];assert driver.act('construction-source',path=path)==after
 driver.act('type-source-selection',path=path,text='ßa');typed=source[:position]+'#'+path+'|ßa|ßa|SSA'+source[position+3:];assert driver.act('construction-source',path=path)==typed
 driver.act('undo',path=path);assert driver.act('construction-source',path=path)==after
 driver.act('undo',path=path);assert driver.act('construction-source',path=path)==source
 driver.act('redo',path=path);assert driver.act('construction-source',path=path)==after
 driver.act('redo',path=path);assert driver.act('construction-source',path=path)==typed
 assert driver.act('editor-is-retained',path=path);assert driver.act('editor-size',path=path)==size
 return typed

def main():
 with tempfile.TemporaryDirectory(prefix='embedded-template-browser-',dir=ROOT/'target') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();original=project();part=doc('<h:copy id="body">saved</h:copy>');code='Narrative\n\n```python\nold\n```\n';(root/'project.md').write_text(original);(root/'part.md').write_text(part);(root/'notes.md').write_text(code)
  (root/'.hickory').mkdir();(root/'.hickory/templates.json').write_text(json.dumps({'version':1,'templates':[{'id':'embedded.context','label':'Embedded context','description':'Compiler-bound comments and linked Unicode transforms','language':'python','syntax':'${LINE_COMMENT}${TM_FILENAME}|${1:${TM_SELECTED_TEXT}}|$1|${1/(.*)/${1:/upcase}/}$0'}]}))
  processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};ui=f'http://127.0.0.1:{port}';engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',ui],env=env);processes.append(engine);backend=fixture.origin(engine)
   vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':backend});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html')
   exercise(driver,root,'notes.md',code)
   driver.act('open',path='project.md');dirty=project('draft.py');driver.act('construction-set-source',path='project.md',source=dirty);assert driver.act('construction-source',path='project.md')==dirty
   driver.act('open',path='part.md');dirty_part=doc('<h:copy id="body">old</h:copy>');driver.act('construction-set-source',path='part.md',source=dirty_part);typed=exercise(driver,root,'part.md',dirty_part)
   # A physically changed entry must not renew an already reviewed proof.
   selected=typed.index('ßa');driver.act('editor-position',path='part.md',anchor=selected,head=selected+2);driver.act('templates-open',path='part.md');driver.act('templates-preview',label='Embedded context');driver.act('templates-hold-apply')
   try:
    (root/'project.md').write_text(original+'\n');refused=driver.act('templates-release-apply');assert refused['status']>=400,refused;assert driver.act('construction-source',path='part.md')==typed
   finally:
    (root/'project.md').write_text(original)
   driver.act('templates-close')
   # The original retained graph, rather than filename or saved output, controls reuse.
   driver.act('open',path='project.md');driver.act('construction-set-source',path='project.md',source=project('draft.py',True));driver.act('open',path='part.md');driver.act('editor-position',path='part.md',anchor=selected,head=selected+2)
   unavailable=driver.act('templates-open',path='part.md');assert 'Preview Embedded context' not in unavailable,unavailable;assert any(word in unavailable.lower() for word in ['language','ambiguous','multiple','disagree']),unavailable;assert driver.act('construction-source',path='part.md')==typed;driver.act('templates-close')
   driver.act('open',path='project.md');claimed=doc('<h:project sources="part.md"/><h:file path="part.md">new generated ownership</h:file>');driver.act('construction-set-source',path='project.md',source=claimed);driver.act('open',path='part.md');driver.act('editor-position',path='part.md',anchor=selected,head=selected+2)
   command=driver.act('templates-command-state')
   if command['disabled']:assert command['reason'],command
   else:
    refused=driver.act('templates-open',path='part.md');assert 'Preview Embedded context' not in refused,refused;assert any(word in refused.lower() for word in ['generat','ownership','output']),refused;driver.act('templates-close')
   assert driver.act('construction-source',path='part.md')==typed
   for path,source in [('project.md',original),('part.md',part),('notes.md',code)]:assert (root/path).read_bytes()==source.encode(),path
   assert all(not(root/name).exists() for name in ['saved.py','draft.py','second.py','mixed.js'])
   print('PASS: real code-fence and twice-reused external fragment catalog/Preview/Apply, dirty original compiler graph, Python comments/authored filenames, Unicode transform mirrors, atomic UndoRedo, retained editors/geometry, old entry proof/ambiguous reuse/new generated location refusal, unchanged source disks and no generated outputs')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()

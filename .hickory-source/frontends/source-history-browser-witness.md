<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Exercise confirmed copies and readable local history in an owned browser workspace

For maintainers checking the complete browser path against the real engine. The
fixture owns every file, process and retained history record; it never reads this
checkout’s regression state. A literal Unicode filename checks encoded routing.
The same retained editor first saves, reads full before/after snapshots, restores
only its draft and exercises both undo mechanisms. Changing the caret retires an
old restore review. Save As must preserve that original editor while publishing
a separate confirmed copy; no later GET can supply publication authority.

<ui:copy id="source-history-browser-real-engine">
"""Owned literal source: confirmed copies and draft-only local-history restoration."""
import hashlib,importlib.util,json,os,socket,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-browser-source-history-') as temporary:
  folder=Path(temporary);root=(folder/'workspace');root.mkdir();path='literal%2F λ?#.py';copy='copy%2F λ?#.py';before=''.join(f'value_{line} = {line}\n' for line in range(100));(root/path).write_text(before);processes=[];driver=None
  environment={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')}
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',f'http://127.0.0.1:{port}'],env=environment);processes.append(engine);backend=fixture.origin(engine)
   vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**environment,'HICKORY_API_ORIGIN':backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path=path);driver.act('remember-editor',path=path);size=driver.act('editor-size',path=path)
   driver.act('source-tests-edit',path=path,text='\n# confirmed saved after\n');after=driver.act('construction-source',path=path);driver.act('save');driver.act('saved-status');assert (root/path).read_text()==after
   driver.act('source-tests-edit',path=path,text='\n# retained unsaved draft\n');draft=driver.act('construction-source',path=path);position=driver.act('editor-position',path=path,anchor=7,head=12,y=240)
   text=driver.act('local-history-open');assert 'Local source history' in text,text
   previous=driver.act('local-history-select',side='before');assert previous['text']==before and previous['draft']==draft,previous
   later=driver.act('local-history-select',id=previous['id'],side='after');assert later['text']==after and later['draft']==draft
   assert driver.act('editor-position',path=path)==position
   assert driver.act('editor-is-retained',path=path) and driver.act('editor-size',path=path)==size
   driver.act('local-history-select',id=previous['id'],side='before');driver.act('local-history-restore');driver.act('local-history-close');assert driver.act('construction-source',path=path)==before;assert (root/path).read_text()==after
   driver.act('undo',path=path);assert driver.act('construction-source',path=path)==draft
   driver.act('redo',path=path);assert driver.act('construction-source',path=path)==before
   driver.act('workspace-history',redo=False);assert driver.act('construction-source',path=path)==draft
   driver.act('workspace-history',redo=True);assert driver.act('construction-source',path=path)==before
   # A changed caret invalidates a previously captured restore, without touching the draft.
   driver.act('local-history-open');driver.act('local-history-select',id=previous['id'],side='before');driver.act('editor-position',path=path,anchor=4);stale=driver.act('local-history-select',id=previous['id'],side='before');assert not stale['enabled'];driver.act('local-history-close');assert driver.act('construction-source',path=path)==before
   # Save As publishes one complete copy and keeps this exact source editor/undo.
   driver.act('file-command',command='Save As',path=copy);driver.act('source-copy-ready',path=copy);assert (root/copy).read_text()==before and (root/path).read_text()==after
   driver.act('file-command',command='Open File',path=path);assert driver.act('editor-is-retained',path=path);assert driver.act('construction-source',path=path)==before
   driver.act('undo',path=path);assert driver.act('construction-source',path=path)==draft;assert (root/path).read_text()==after and (root/copy).read_text()==before
   assert not driver.act('debug-diagnostics')['errors']
   print('PASS: strict literal source Save/Save As; direct confirmed copy; existing editor/undo retained; full before/after local history; draft-only restore; native/workspace Undo/Redo; stale selection refusal; code geometry and saved bytes preserved')
  finally:
   if driver:driver.close()
   for process in reversed(processes):
    process.terminate()
    try:process.wait(timeout=5)
    except Exception:process.kill();process.wait()
if __name__=='__main__':main()
</ui:copy>

This is browser automation with a real local engine, not physical keyboard,
touch, screen-reader or native accessibility acceptance. The fixture checks
only its selected behavior and safety paths.

<ui:file path="scripts/test-source-history-browser.py"><ui:paste select="#source-history-browser-real-engine"/></ui:file>
## Focused execution evidence

The canonical browser witness passed against the real local engine on 2026-11-16:
strict Save and Save As for the literal Unicode path, direct confirmed copy
adoption, retained original editor/undo, complete before/after previews,
draft-only restoration, native and workspace Undo/Redo, stale-selection refusal,
unchanged editor geometry and expected saved bytes. The focused canonical suite
passed 13 copy/File-menu cases and 22 history/save/conflict cases; TypeScript
validation passed. The browser flow does not establish physical input,
accessibility, alternate-browser behavior, or native platform acceptance.
Namespace, source and receipt refusal additionally have focused model checks;
this browser witness does not simulate replacing a live filesystem root.

</ui:doc>

<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Verify authored bytes in the mounted browser editor

For maintainers checking that display normalization never becomes source authority.

The owned fixture writes CRLF and mixed CRLF/LF bytes directly, opens them in the actual simple UI, and types through its editor. It verifies local Enter, a raw mixed clipboard event, retained selection and native undo/redo. A genuine TypeScript extraction after CRLF-prefixed lines checks authored provider offsets, source proofs and coordinated workspace undo. The saved file bytes must stay unchanged throughout. Automation hooks expose editor state only to the fixture; they do not establish physical-input or accessibility acceptance.

<ui:copy id="browser-source-text-witness">
"""Owned browser acceptance for exact authored line endings and provider offsets."""
import importlib.util,json,os,socket,tempfile,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SOURCES={'crlf.ts':'// first\r\nfunction greet(name: string) {\r\n  return "Hello " + name;\r\n}\r\nconsole.log(greet("Ada"));\r\n','mixed.ts':'// first\r\nconst two = 2;\nconst three = 3;\r\nconsole.log(two + three);\n'}
def main():
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 tools=ROOT/'target/refactor-provider/node_modules';server=tools/'typescript-language-server/lib/cli.mjs';assert server.is_file()
 with tempfile.TemporaryDirectory(prefix='hickory-source-text-') as temporary:
  folder=Path(temporary).resolve();root=folder/'workspace';root.mkdir();driver=None;processes=[]
  for path,text in SOURCES.items():(root/path).write_bytes(text.encode())
  (root/'package.json').write_text('{"type":"module"}');(root/'tsconfig.json').write_text('{"compilerOptions":{"strict":true,"target":"ES2022","ignoreDeprecations":"6.0"}}');(root/'node_modules').mkdir();(root/'node_modules/typescript').symlink_to(tools/'typescript',target_is_directory=True);(root/'.hick-lsp.json').write_text(json.dumps({'servers':{'typescript':{'command':['node',str(server),'--stdio']}}}))
  with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
  env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};engine=fixture.start([str(ROOT/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window',f'http://127.0.0.1:{port}'],env=env);processes.append(engine)
  try:
   origin=fixture.origin(engine);vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html')
   for path,source in SOURCES.items():
    driver.act('open',path=path);driver.act('remember-editor',path=path);opened=driver.act('source-text-state',path=path);assert opened['source']==source and opened['display']==source.replace('\r\n','\n') and not opened['undo'],opened
    at=source.index('const') if path=='mixed.ts' else source.index('function');driver.act('source-text-select',path=path,anchor=at,head=at);driver.act('source-text-type',path=path,text='/* dirty */');typed=driver.act('source-text-state',path=path);assert typed['source']==source[:at]+'/* dirty */'+source[at:];driver.act('undo',path=path);assert driver.act('source-text-state',path=path)['source']==source;driver.act('redo',path=path);assert driver.act('source-text-state',path=path)['source']==typed['source'];driver.act('undo',path=path)
    driver.act('source-text-select',path=path,anchor=0,head=0);driver.act('source-text-type',path=path,key='Enter');entered=driver.act('source-text-state',path=path);assert entered['source']=='\r\n'+source,entered;driver.act('undo',path=path);assert driver.act('source-text-state',path=path)['source']==source
    at=source.index('console');driver.act('source-text-select',path=path,anchor=at,head=at);driver.act('source-text-paste',path=path,text='// pasted\n// mixed\r\n');pasted=driver.act('source-text-state',path=path);assert pasted['source']==source[:at]+'// pasted\n// mixed\r\n'+source[at:];driver.act('undo',path=path);assert driver.act('source-text-state',path=path)['source']==source;assert driver.act('editor-is-retained',path=path)
   path='crlf.ts';source=SOURCES[path];driver.act('open',path=path);selected='"Hello " + name';at=source.index(selected);driver.act('source-text-select',path=path,anchor=at,head=at+len(selected));position=driver.act('source-text-state',path=path);size=driver.act('editor-size',path=path);driver.act('change-open',command='Refactor and quick fixes');deadline=time.monotonic()+35;title="Extract to inner function in function 'greet'"
   while title not in (reading:=driver.act('change-prepare',command='Prepare provider actions')):
    assert time.monotonic()<deadline,reading;time.sleep(.1)
   assert driver.act('source-text-state',path=path)==position,'Provider attachment changed source, selection or history';driver.act('action-choose',title=title);driver.act('change-apply');driver.act('change-close');changed=driver.act('source-text-state',path=path);assert 'function newFunction' in changed['source'];assert changed['source'].count('\r\n')>=source.count('\r\n'),'Unchanged original separators were normalized';assert driver.act('editor-is-retained',path=path) and driver.act('editor-size',path=path)==size
   driver.act('change-open',command='Undo workspace change');driver.act('change-close');assert driver.act('source-text-state',path=path)['source']==source;driver.act('change-open',command='Redo workspace change');driver.act('change-close');assert driver.act('source-text-state',path=path)['source']==changed['source'];assert not driver.act('debug-diagnostics')['errors']
   for path,text in SOURCES.items():assert (root/path).read_bytes()==text.encode(),path
   print('PASS rendered browser: byte-exact CRLF/mixed opening, typing/local Enter/raw paste/native UndoRedo; authored UTF16 selection and genuine provider refactor/proofs/workspace UndoRedo; async services preserve editor identity/geometry/source. No source save. Physical input/accessibility remain unverified.',flush=True)
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-browser-source-text.py"><ui:paste select="#browser-source-text-witness"/></ui:file>

</ui:doc>

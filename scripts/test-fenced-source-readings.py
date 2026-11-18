"""Owned genuine provider fixture for ordinary authored Markdown code readings."""
import argparse,importlib.util,json,os,socket,subprocess,sys,tempfile,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SOURCE='Readable prose 😀\r\n\r\n  ```python\r\n  # Unicode 😀\r\n  def greet(name: str) -> str:\r\n      return name.upper()\r\n  result = greet("reader")\r\n  ```\r\n\r\n```typescript\r\nconst marker = "😀"; function greet(name: string) { return name.toUpperCase(); }\r\nconst result = greet("reader");\r\n```\r\n'
ROWS=[{'query':{'line':6,'character':12},'definition':{'line':4,'character':6}}, {'query':{'line':11,'character':16},'definition':{'line':10,'character':30}}]
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--host',choices=['protocol','browser','apple','android'],default='protocol');args=parser.parse_args()
 spec=importlib.util.spec_from_file_location('fenced_simple_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 tools=ROOT/'target/refactor-provider/node_modules';typescript=tools/'typescript-language-server/lib/cli.mjs';compiler=tools/'typescript/lib/tsserver.js';python=ROOT/'target/fenced-lsp-tools/basedpyright/dist/pyright-langserver.js'
 for path in [typescript,compiler,python]:assert path.is_file(),f'Pinned installed provider missing: {path}'
 assert json.loads((tools/'typescript-language-server/package.json').read_text())['version']=='6.1.0'
 assert json.loads((tools/'typescript/package.json').read_text())['version']=='6.0.3'
 with tempfile.TemporaryDirectory(prefix='fenced-source-reading-',dir=ROOT/'target') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'notes.md').write_bytes(SOURCE.encode());(root/'.hickory-native-template-fixture').write_text('owned-native-templates-v1\n')
  (root/'node_modules').mkdir();(root/'node_modules/typescript').symlink_to(tools/'typescript',target_is_directory=True)
  (root/'tsconfig.json').write_text(json.dumps({'compilerOptions':{'strict':True,'noEmit':True},'typeAcquisition':{'enable':False}}))
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':['node',str(python),'--stdio']},'typescript':{'command':['node',str(typescript),'--stdio']}}}))
  data=folder/'fixture.json';data.write_text(json.dumps({'source':SOURCE,'rows':ROWS}));processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',f'http://127.0.0.1:{port}'],env=env);processes.append(engine);origin=fixture.origin(engine)
   if args.host=='protocol':subprocess.run(['node',str(ROOT/'scripts/fenced-source-reading-driver.mjs'),origin,str(data)],cwd=ROOT,check=True,timeout=180)
   elif args.host in ['apple','android']:
    spec=importlib.util.spec_from_file_location('native_fixture',ROOT/'scripts/test-native-templates.py');native=importlib.util.module_from_spec(spec);spec.loader.exec_module(native)
    native.run_host(args.host,origin,root,folder,fixture,acceptance=(('FencedSourceReadingTests','testLiveFencedSourceLanguageReadings','fenced-source') if args.host=='apple' else ('FencedSourceReadingTest','liveFencedSourceLanguageReadings','fenced-source')))
   else:
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path='notes.md');driver.act('remember-editor',path='notes.md');size=driver.act('editor-size',path='notes.md')
    for row in ROWS:
     lines=SOURCE.splitlines(keepends=True);prefix=''.join(lines[:row['query']['line']])+lines[row['query']['line']][:row['query']['character']];offset=len(prefix.replace('\r\n','\n'));deadline=time.monotonic()+30
     while True:
      result=driver.act('semantic-query',path='notes.md',offset=offset,query='Go to definition');text=result['text']
      if 'notes.md' in text and '1 locations' in text:break
      driver.act('semantic-close');assert time.monotonic()<deadline,text;time.sleep(.1)
     assert driver.act('construction-source',path='notes.md')==SOURCE;definition=row['definition'];driver.act('semantic-open',path='notes.md',label=f"notes.md:{definition['line']+1}:{definition['character']+1}");before=''.join(lines[:definition['line']]);expected=len(before.encode('utf-16-le'))//2+definition['character'];assert driver.act('editor-selections',path='notes.md')['ranges']==[{'from':expected,'to':expected}]
    assert driver.act('editor-is-retained',path='notes.md');assert driver.act('editor-size',path='notes.md')==size
   assert(root/'notes.md').read_bytes()==SOURCE.encode()
   assert set(path.name for path in root.iterdir())=={'notes.md','.hickory-native-template-fixture','tsconfig.json','.hick-lsp.json','node_modules'},'Language readings materialized unexpected files'
   print('PASS: genuine providers read authored fences; saved source bytes and workspace file roster unchanged. Physical-input and accessibility acceptance remain unverified.')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()

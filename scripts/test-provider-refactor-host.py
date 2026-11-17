#!/usr/bin/env python3
"""Genuine edits-only TypeScript refactoring through one simple host."""
import argparse,importlib.util,json,os,re,socket,subprocess,tempfile,time,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
CASES=[
 {'path':'extract.ts','source':'function greet(name: string) { return "Hello " + name; }\nconsole.log(greet("Ada"));\n','selection':'"Hello " + name','title':"Extract to inner function in function 'greet'",'expected':'function newFunction'},
 {'path':'accessor.ts','source':'class Person { name = "Ada"; }\nconst person = new Person();\nconsole.log(person.name);\n','selection':'name','title':"Generate 'get' and 'set' accessors",'expected':'get name()'},
]
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['browser','apple','android']);parser.add_argument('--skip-build',action='store_true');args=parser.parse_args()
 tools=ROOT/'target/refactor-provider/node_modules';server=tools/'typescript-language-server/lib/cli.mjs';compiler=tools/'typescript/lib/tsc.js'
 for package,version in [('typescript-language-server','6.1.0'),('typescript','6.0.3')]:assert json.loads((tools/package/'package.json').read_text())['version']==version
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-refactor-host-') as temporary:
  folder=Path(temporary).resolve();root=folder/'workspace';root.mkdir();artifacts=folder/'drafts';artifacts.mkdir();processes=[];driver=None
  for case in CASES:(root/case['path']).write_text(case['source'])
  (root/'.hickory-refactor-fixture').write_text('owned-refactor-v1\n');(root/'.hickory-refactor-cases.json').write_text(json.dumps(CASES));(root/'package.json').write_text('{"type":"module"}');(root/'tsconfig.json').write_text('{"compilerOptions":{"strict":true,"target":"ES2022","ignoreDeprecations":"6.0"}}')
  (root/'node_modules').mkdir();(root/'node_modules/typescript').symlink_to(tools/'typescript',target_is_directory=True)
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'typescript':{'command':['node',str(server),'--stdio']}}}))
  env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')}
  with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
  engine=fixture.start([str(ROOT/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window',f'http://127.0.0.1:{port}'],env=env);processes.append(engine)
  try:
   origin=fixture.origin(engine)
   if args.host=='browser':
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html')
    for case in CASES:
     path=case['path'];draft=case['source']+'// retained dirty\n';start=draft.index(case['selection']);driver.act('open',path=path);driver.act('remember-editor',path=path);driver.act('construction-set-source',path=path,source=draft);position=driver.act('editor-position',path=path,anchor=start,head=start+len(case['selection']));size=driver.act('editor-size',path=path);driver.act('change-open',command='Refactor and quick fixes');deadline=time.monotonic()+30
     while case['title'] not in (reading:=driver.act('change-prepare',command='Prepare provider actions')):
      assert time.monotonic()<deadline,reading;time.sleep(.05)
     driver.act('action-choose',title=case['title']);assert driver.act('construction-source',path=path)==draft;assert driver.act('editor-position',path=path)==position;driver.act('change-apply');driver.act('change-close');changed=driver.act('construction-source',path=path);assert case['expected'] in changed and '// retained dirty' in changed,changed;(artifacts/path).write_text(changed)
     driver.act('change-open',command='Undo workspace change');driver.act('change-close');assert driver.act('construction-source',path=path)==draft;driver.act('change-open',command='Redo workspace change');driver.act('change-close');assert driver.act('construction-source',path=path)==changed;assert driver.act('editor-is-retained',path=path) and driver.act('editor-size',path=path)==size
   else:
    env.update(HICKORY_REFACTOR_ORIGIN=origin,HICKORY_REFACTOR_WORKSPACE=str(root),HICKORY_REFACTOR_ARTIFACTS=str(artifacts));name='testGenuineExtractAndGenerationKeepDirtyEditorAndHistory' if args.host=='apple' else 'genuineExtractAndGenerationKeepDirtyEditorAndHistory';report=folder/'native.xml'
    if args.host=='apple':command=['swift','test','--jobs','2','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','ProviderRefactorTests.'+name+'$','--xunit-output',str(report)]+(['--skip-build'] if args.skip_build else [])
    else:report=ROOT/'apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.ProviderRefactorTest.xml';command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.ProviderRefactorTest.'+name,'--rerun']
    report.unlink(missing_ok=True);log=ROOT/'target'/f'provider-refactor-{args.host}-actual.log'
    with log.open('w') as output:result=subprocess.run(command,cwd=ROOT,env=env,stdout=output,stderr=subprocess.STDOUT,timeout=600)
    assert result.returncode==0,str(log)
    if report.exists():
     rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(key) is not None for key in ['failure','error','skipped']),str(report)
    else:assert args.host=='apple' and 'Executed 1 test, with 0 failures' in log.read_text() and re.search(re.escape(name)+r'.*passed',log.read_text()) and 'Test skipped' not in log.read_text()
   for case in CASES:
    assert (root/case['path']).read_text()==case['source'];assert case['expected'] in (artifacts/case['path']).read_text()
   result=subprocess.run(['node',str(compiler),'--noEmit','--strict','--target','ES2022','--ignoreConfig',*[str(artifacts/case['path']) for case in CASES]],capture_output=True,text=True,timeout=60);assert result.returncode==0,result.stdout+result.stderr
   assert not (root/'.hick-cache').exists();print('PASS '+args.host+': genuine extract and accessor generation, edits-only preview, retained dirty editor/history, unchanged saved sources and TypeScript-checked resulting drafts; physical input/accessibility remains unverified.')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()

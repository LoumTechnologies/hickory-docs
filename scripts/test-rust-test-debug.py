"""Owned real Cargo/LLDB source-test Debug flows. No physical-input claim."""
import argparse,importlib.util,json,os,socket,tempfile,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def module(path,name):
 spec=importlib.util.spec_from_file_location(name,path);value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value
def wait(test,label):
 deadline=time.monotonic()+45
 while time.monotonic()<deadline:
  if test():return
  time.sleep(.05)
 raise RuntimeError(label)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--host',choices=['browser','apple','android'],required=True);args=parser.parse_args();fixture=module(ROOT/'scripts/test-simple-browser.py','simple_fixture');native=module(ROOT/'scripts/test-native-templates.py','native_fixture')
 with tempfile.TemporaryDirectory(prefix='rust-selected-debug-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.git').mkdir();(root/'.hickory').mkdir();(root/'.hickory-rust-test-debug-fixture').write_text('owned-rust-test-debug-v1\n');sibling=json.dumps(str(root/'sibling-ran'));source=f'''#[cfg(test)]
mod cases {{
 #[test]
 fn test_selected() {{
  let selected_value=41;
  assert_eq!(selected_value,41); // owned-rust-stop
 }}
 #[test]
 fn test_sibling() {{ std::fs::write({sibling},b"wrong").unwrap(); }}
}}
''';manifest='[workspace]\n[package]\nname="owned-selected-debug"\nversion="0.1.0"\nedition="2024"\n';(root/'physical/src').mkdir(parents=True);(root/'physical/Cargo.toml').write_text(manifest);(root/'physical/src/lib.rs').write_text(source);authored=f'<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="Cargo.toml">\n{manifest}</h:file>\n<h:file path="src/lib.rs">\n{source}</h:file>\n</h:doc>\n';(root/'tests.md').write_text(authored);processes=[];driver=None
  try:
   with socket.socket() as reserve:reserve.bind(('127.0.0.1',0));port=reserve.getsockname()[1]
   ui=f'http://127.0.0.1:{port}';env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',ui],env=env);processes.append(engine);origin=fixture.origin(engine)
   if args.host=='browser':
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html')
    for path,text in [('physical/src/lib.rs',source),('tests.md',authored)]:
     driver.act('file-command',command='Open File',path=path);driver.act('remember-editor',path=path);geometry=driver.act('editor-size',path=path);before=driver.act('source-text-state',path=path);line=next(i+1 for i,row in enumerate(text.splitlines())if'owned-rust-stop'in row);driver.act('debug-breakpoint',line=line);driver.act('source-tests-open');driver.act('source-tests-read');driver.act('source-tests-debug',filter='test_selected');driver.act('source-tests-debug-link');wait(lambda:any('paused'in row.lower()for row in driver.act('debug-state')),'Selected Rust test did not pause');assert not(root/'sibling-ran').exists();paused=driver.act('source-text-state',path=path);assert all(paused[key]==before[key] for key in ['source','undo','redo']);driver.act('debug-breakpoint-remove',path=path);driver.act('debug-continue');wait(lambda:any('finished'in row.lower()for row in driver.act('debug-state')),'Selected Rust test did not finish');driver.act('source-tests-open');records=driver.act('source-tests-debug-records');assert'process exit 0'in records and'test_selected'in records;driver.act('source-tests-close');assert driver.act('source-text-state',path=path)==paused;assert driver.act('construction-source',path=path)==text;assert(root/path).read_text()==text;assert driver.act('editor-is-retained',path=path);assert driver.act('editor-size',path=path)==geometry;assert not(root/'sibling-ran').exists();assert not(root/'Cargo.toml').exists();assert not(root/'src/lib.rs').exists()
    assert not driver.act('debug-diagnostics')['errors'];print('PASS browser: saved physical/authored Rust test selection through real Cargo and LLDB, sibling exclusion, retained source/geometry/history')
   else:
    acceptance=('RustSelectedTestDebugTests','testLiveSavedRustSelectionAndHistory','rust-selected-test')if args.host=='apple'else('RustSelectedTestDebugTest','liveSavedRustSelectionAndHistory','rust-selected-test');native.run_host(args.host,origin,root,folder,fixture,acceptance=acceptance)
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()

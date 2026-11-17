#!/usr/bin/env python3
"""Owned production native bound rename/format preview and draft-history flow."""
import argparse,importlib.util,json,os,re,signal,subprocess,sys,tempfile,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SOURCE='value = 1\n'
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['apple','android']);parser.add_argument('--skip-build',action='store_true');parser.add_argument('--literate',action='store_true');args=parser.parse_args()
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 def interrupted(number,_frame):raise SystemExit(128+number)
 signal.signal(signal.SIGTERM,interrupted)
 with tempfile.TemporaryDirectory(prefix='hickory-native-bound-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.hickory-native-bound-fixture').write_text('owned-native-bound-v1\n');events=folder/'events';gate=folder/'gate'
  if args.literate:
   source_spec=importlib.util.spec_from_file_location('literate_fixture',ROOT/'scripts/test-provider-bulk-literate.py');source_module=importlib.util.module_from_spec(source_spec);source_spec.loader.exec_module(source_module);sources=source_module.SOURCE;provider='controlled-bulk-literate-provider.py'
  else:sources={'a.py':SOURCE};provider='controlled-bound-source-provider.py'
  for path,text in sources.items():(root/path).write_text(text)
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':[sys.executable,str(ROOT/'scripts'/provider),str(events),str(gate)]}}}))
  env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};engine=fixture.start([str(ROOT/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window','http://127.0.0.1:1'],env=env)
  try:
   env.update(HICKORY_NATIVE_BOUND_ORIGIN=fixture.origin(engine),HICKORY_NATIVE_BOUND_WORKSPACE=str(root),HICKORY_NATIVE_BOUND_EVENTS=str(events),HICKORY_NATIVE_BOUND_GATE=str(gate))
   if args.host=='apple':
    name='testLiveOwnedLiterateBulkFixAcquiresContributorAndKeepsProseEditor' if args.literate else 'testLiveOwnedBoundRenameFormatPreviewUndoAndStaleRefusal';command=['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','ActionChoiceTests.'+name+'$']+(['--skip-build'] if args.skip_build else [])
   else:
    name='liveOwnedLiterateBulkFixAcquiresContributorAndKeepsProseEditor' if args.literate else 'liveOwnedBoundRenameFormatPreviewUndoAndStaleRefusal';command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.ActionChoiceTest.'+name,'--rerun']
   result=subprocess.run(command,cwd=ROOT,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True);print(result.stdout[-7000:],end='');assert result.returncode==0,result.returncode
   if args.host=='apple':assert re.search(re.escape(name)+r'.*passed',result.stdout) and 'Executed 1 test, with 0 failures' in result.stdout and 'Test skipped' not in result.stdout
   else:
    report=ROOT/'apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.ActionChoiceTest.xml';rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(kind) is not None for kind in ['skipped','failure','error'])
   for path,text in sources.items():assert (root/path).read_text()==text,path
   messages=[json.loads(line) for line in events.read_text().splitlines()]
   if args.literate:
    selected=[row for row in messages if row.get('query','').endswith('/out/a.py')];assert len(selected)>=2 and all(row.get('whole') is True for row in selected),messages;assert any(row.get('original') is True for row in messages);assert not (root/'out').exists()
    print(f'PASS {args.host} literate: prose-start complete-file fix acquired copy contributor, requeried current graph, previewed/applied/Undid/Redid source drafts, retained original editor/selection and all saved Markdown bytes, no generated output materialized. Controlled provider protocol is not language semantics or physical/accessibility evidence.');return
   assert any(row.get('method')=='textDocument/rename' and row.get('name')=='native_renamed' for row in messages);assert any(row.get('method')=='textDocument/formatting' for row in messages);assert any(row.get('held')=='held_native' for row in messages)
   print(f'PASS {args.host}: production bound rename/format preview, source-draft apply, coordinated Undo/Redo, retained editor and unchanged disk; late equal-byte edit/undo retires held proposal. Controlled provider protocol does not certify language semantics or physical/accessibility input.')
  finally:fixture.stop(engine)
if __name__=='__main__':main()

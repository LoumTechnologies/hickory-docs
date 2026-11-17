#!/usr/bin/env python3
"""Owned provider-to-native complete-file fix review and coordinated draft history."""
import argparse,importlib.util,json,os,re,subprocess,sys,tempfile,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['apple','android']);parser.add_argument('--skip-build',action='store_true');args=parser.parse_args()
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-native-bulk-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.hickory-bulk-fixture').write_text('owned-bulk-v1\n');source='value = 1\n'
  for path in ['a.py','other.py']:(root/path).write_text(source)
  events=folder/'events.jsonl';(root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':[sys.executable,str(ROOT/'scripts/controlled-action-provider.py'),'python',str(events)]}}}))
  env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state'),'HICKORY_CONTROLLED_ACTION_OTHER':'1'}
  engine=fixture.start([str(ROOT/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window','http://127.0.0.1:1'],env=env)
  try:
   env.update(HICKORY_BULK_ORIGIN=fixture.origin(engine),HICKORY_BULK_WORKSPACE=str(root))
   if args.host=='apple':
    name='testLiveOwnedBulkFixCommandReviewApplyAndUndo';command=['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','ActionChoiceTests.'+name+'$']+(['--skip-build'] if args.skip_build else [])
   else:
    name='liveOwnedBulkFixCommandReviewApplyAndUndo';command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.ActionChoiceTest.'+name,'--rerun']
   log=ROOT/'target'/f'native-bulk-{args.host}.log'
   with log.open('w') as output:result=subprocess.run(command,cwd=ROOT,env=env,stdout=output,stderr=subprocess.STDOUT)
   if result.returncode:raise RuntimeError(f'{args.host} failed: {log}')
   if args.host=='apple':
    text=log.read_text();assert re.search(re.escape(name)+r'.*passed',text) and 'Executed 1 test, with 0 failures' in text and 'Test skipped' not in text,str(log)
   else:
    report=ROOT/'apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.ActionChoiceTest.xml';rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(key) is not None for key in ['skipped','failure','error']),str(report)
   records=[json.loads(line) for line in events.read_text().splitlines()];queries=[record for record in records if record.get('bulk_query')=='python'];assert len(queries)>=2 and all(record['context'].get('only')==['source.fixAll'] and record['range']=={'start':{'line':0,'character':0},'end':{'line':1,'character':0}} for record in queries),records
   assert len([record for record in records if record.get('resolved')=='python'])==2,records
   for path in ['a.py','other.py']:assert (root/path).read_text()==source
   print(f'PASS {args.host}: real provider bulk family, existing command, complete two-file preview/reacquisition, draft Apply/Undo/Redo, retained editor state and unchanged disk; no skipped acceptance. Physical input/accessibility unverified.')
  finally:fixture.stop(engine)
if __name__=='__main__':main()

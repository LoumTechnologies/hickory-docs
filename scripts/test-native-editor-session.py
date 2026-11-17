"""Owned two-process native editor checkpoint/restart recovery acceptance."""
import argparse,importlib.util,json,os,re,signal,subprocess,tempfile,threading,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['apple','android']);parser.add_argument('--serve-only',action='store_true');parser.add_argument('--skip-build',action='store_true');args=parser.parse_args()
 def interrupted(number,_frame):raise SystemExit(128+number)
 signal.signal(signal.SIGTERM,interrupted)
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-native-session-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.hickory-native-session-fixture').write_text('owned-native-session-v1\n');(root/'files').write_text(''.join(f'line {i} = {i}\n' for i in range(150)));(root/'note.py').write_text('value=1\n')
  engine=fixture.start([str(ROOT/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')})
  try:
   origin=fixture.origin(engine);print(json.dumps({'origin':origin,'workspace':str(root),'acceptance':'not run'}),flush=True)
   if args.serve_only:threading.Event().wait();return
   for phase in ['capture','restore']:
    env={**os.environ,'HICKORY_NATIVE_SESSION_ORIGIN':origin,'HICKORY_NATIVE_SESSION_WORKSPACE':str(root),'HICKORY_NATIVE_SESSION_PHASE':phase}
    if args.host=='apple':
     name='testLiveOwnedProcessRestartRecovery';command=['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','EditorRecoveryTests.'+name+'$']+(['--skip-build'] if args.skip_build else [])
    else:
     name='liveOwnedProcessRestartRecovery';command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.EditorRecoveryTest.'+name,'--rerun-tasks']
    log=ROOT/'target'/f'native-session-{args.host}-{phase}.log'
    with log.open('w') as output:result=subprocess.run(command,cwd=ROOT,env=env,stdout=output,stderr=subprocess.STDOUT)
    if result.returncode:raise RuntimeError(f'{args.host} {phase} failed; inspect {log}')
    if args.host=='apple':
     text=log.read_text();assert re.search(re.escape(name)+r'.*passed',text) and re.search(r'Executed 1 test, with 0 failures',text) and 'Test skipped' not in text,str(log)
    else:
     report=ROOT/'apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.EditorRecoveryTest.xml';rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(key) is not None for key in ['skipped','failure','error']),str(report)
    print(f'PASS {args.host} {phase}: exact owned process-restart case, no skips',flush=True)
   assert (root/'note.py').read_text()=='value=1\n' and 'unsaved' not in (root/'files').read_text()
   print(f'PASS {args.host}: separate-process automatic checkpoint and draft/layout/view recovery; existing editor/Undo retained; saved source unchanged. Physical input/accessibility remains unverified.',flush=True)
  finally:fixture.stop(engine)
if __name__=='__main__':main()

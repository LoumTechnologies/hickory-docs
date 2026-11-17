#!/usr/bin/env python3
"""Owned real pytest collector flows through the compact host adapters."""
import argparse,importlib.util,json,os,re,signal,socket,subprocess,sys,tempfile,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
APP='def classify(value):\n    if value < 0:\n        return -1\n    return 1\n'
TEST='from pathlib import Path\nfrom app import classify\ndef record(name):\n    with Path(".hick-cache/executions").open("a") as out: out.write(name+"\\n")\ndef test_positive():\n    record("positive")\n    assert classify(1) == 1\ndef test_negative():\n    record("negative")\n    assert classify(-1) == 1\n'
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['browser','apple','android']);parser.add_argument('--python-env',type=Path,default=ROOT/'target/thread-debug-env');parser.add_argument('--skip-build',action='store_true');args=parser.parse_args()
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 def interrupted(number,_frame):raise SystemExit(128+number)
 signal.signal(signal.SIGTERM,interrupted)
 with tempfile.TemporaryDirectory(prefix='hickory-collector-host-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.hickory-collector-fixture').write_text('owned-collector-v1\n');(root/'app.py').write_text(APP);(root/'test_app.py').write_text(TEST);(root/'notes.py').write_text('note = 1\n');(root/'pyproject.toml').write_text('[project]\nname="collector-fixture"\nversion="0.0.0"\n[tool.pytest.ini_options]\njunit_family="xunit1"\n');(root/'.venv').symlink_to(args.python_env.resolve(),target_is_directory=True)
  env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};processes=[]
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   ui=f'http://127.0.0.1:{port}';engine=fixture.start([str(ROOT/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window',ui],env=env);processes.append(engine);origin=fixture.origin(engine);env.update(HICKORY_COLLECTOR_ORIGIN=origin,HICKORY_COLLECTOR_WORKSPACE=str(root))
   if args.host=='browser':
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);url=fixture.origin(vite)+'/simple.html';command=['node',str(ROOT/'scripts/collector-browser-driver.mjs'),url]
   elif args.host=='apple':
    name='testLiveOwnedCollectorSurfaceResultsRerunAndSourceGuards';command=['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','CoverageViewTests.'+name+'$']+(['--skip-build'] if args.skip_build else [])
   else:
    name='liveOwnedCollectorBridgeResultsRerunAndSourceGuards';command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.CoverageBindingTest.'+name,'--rerun']
   result=subprocess.run(command,cwd=ROOT,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True);print(result.stdout[-6000:],end='');assert result.returncode==0,result.returncode
   if args.host=='apple':assert re.search(re.escape(name)+r'.*passed',result.stdout) and 'Executed 1 test, with 0 failures' in result.stdout and 'Test skipped' not in result.stdout
   if args.host=='android':
    report=ROOT/'apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.CoverageBindingTest.xml';rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(k) is not None for k in ['skipped','failure','error'])
   assert (root/'app.py').read_text()==APP and (root/'test_app.py').read_text()==TEST and (root/'notes.py').read_text()=='note = 1\n';events=(root/'.hick-cache/executions').read_text().splitlines();assert sorted(events)==['negative','negative','positive'],events
   print(f'PASS {args.host}: actual pytest discovery/run/failed coverage/rerun-failed/history, unchanged source and retained UI state. Physical input/accessibility remains unverified.')
  finally:
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()

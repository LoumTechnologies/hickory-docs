#!/usr/bin/env python3
"""Owned deep-stack and source-edit breakpoint retention acceptance for simple hosts."""
import argparse,importlib.util,json,os,re,signal,socket,subprocess,tempfile,time,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SOURCE="def descend(depth):\n    token = depth\n    if depth == 0:\n        marker = 41  # deep-stop\n        return marker\n    return descend(depth - 1)\n\ndescend(130)\n"
EDIT="# retained unsaved breakpoint edit\n"
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['browser','apple','android']);parser.add_argument('--skip-build',action='store_true');parser.add_argument('--engine',type=Path,default=ROOT/'target/source-edit-focus/debug/examples/engine_client');parser.add_argument('--venv',type=Path,default=ROOT/'target/thread-debug-env');args=parser.parse_args()
 def interrupted(number,_frame):raise SystemExit(128+number)
 signal.signal(signal.SIGTERM,interrupted)
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 python=args.venv.resolve()/'bin/python';assert python.exists(),'An owned prepared debugpy environment is required; use the native-debug fixture setup.'
 def wait(read,reason):
  deadline=time.monotonic()+40
  while time.monotonic()<deadline:
   if read():return
   time.sleep(.03)
  raise RuntimeError(reason)
 with tempfile.TemporaryDirectory(prefix='hickory-debug-retention-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.git').mkdir();(root/'.venv').symlink_to(args.venv.resolve(),target_is_directory=True);(root/'.hickory-debug-retention-fixture').write_text('owned-debug-retention-v1\n');(root/'deep.py').write_text(SOURCE);(root/'project.md').write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources=""/></h:doc>');(root/'.hick-launch.json').write_text(json.dumps({'version':1,'profiles':[{'id':'owned-deep','name':'Owned deep stack','program':'deep.py','default':True}]}))
  env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   engine=fixture.start([str(args.engine.resolve()),str(root),'window',f'http://127.0.0.1:{port}'],env=env);processes.append(engine);origin=fixture.origin(engine)
   env.update(HICKORY_DEBUG_RETENTION_ORIGIN=origin,HICKORY_DEBUG_RETENTION_WORKSPACE=str(root))
   if args.host=='browser':
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path='deep.py');driver.act('remember-editor',path='deep.py');geometry=driver.act('editor-size',path='deep.py')
    line=next(index+1 for index,text in enumerate(SOURCE.splitlines()) if '# deep-stop' in text);driver.act('debug-breakpoint',line=line);driver.act('debug-breakpoint-condition',path='deep.py',line=line,condition='depth == 0');driver.act('debug-start',path='deep.py');wait(lambda:any('paused' in text.lower() for text in driver.act('debug-state')),'Deep program did not pause')
    selected=driver.act('debug-frame-index',index=100);assert selected['count']>=131,selected;assert driver.act('debug-watch',expression='depth',expected='100')=='100';assert driver.act('editor-is-retained',path='deep.py') and driver.act('editor-size',path='deep.py')==geometry
    before=[row for row in driver.act('debug-wire') if row.get('op') in ['breakpoints','source_breakpoints']];driver.act('source-tests-edit',path='deep.py',text=EDIT);selection=driver.act('construction-source',path='deep.py',selection=True);driver.act('debug-breakpoint',line=line);reading=driver.act('debug-breakpoint-reading');assert 'Earlier source' in reading and 'depth == 0' in reading,reading;after=[row for row in driver.act('debug-wire') if row.get('op') in ['breakpoints','source_breakpoints']];assert len(after)==len(before),(before,after);assert (root/'deep.py').read_text()==SOURCE;assert driver.act('construction-source',path='deep.py',selection=True)==selection
    driver.act('debug-stop');wait(lambda:not any(any(word in text.lower() for word in ['paused','running','starting','unconfirmed','stopping']) for text in driver.act('debug-state')),'Deep session did not stop');driver.act('file-command',command='Save');wait(lambda:(root/'deep.py').read_text()==SOURCE+EDIT,'Edited source Save was not confirmed');driver.act('debug-breakpoint-condition',path='deep.py',line=line,condition='depth <= 0');driver.act('debug-start',path='deep.py');wait(lambda:any('paused' in text.lower() for text in driver.act('debug-state')),'Edited deep program did not pause');assert driver.act('debug-frame-index',index=100)['count']>=131;assert driver.act('debug-watch',expression='depth',expected='100')=='100';driver.act('debug-stop');wait(lambda:not any(any(word in text.lower() for word in ['paused','running','starting','unconfirmed','stopping']) for text in driver.act('debug-state')),'Edited deep session did not stop');driver.act('undo',path='deep.py');assert driver.act('construction-source',path='deep.py')==SOURCE;assert (root/'deep.py').read_text()==SOURCE+EDIT;assert driver.act('editor-is-retained',path='deep.py') and driver.act('editor-size',path='deep.py')==geometry;assert not driver.act('debug-diagnostics')['errors']
   else:
    if args.host=='apple':
     name='testLiveOwnedDeepStackAndEditedBreakpointRetention';command=['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','DebuggerRetentionTests.'+name+'$']+(['--skip-build'] if args.skip_build else [])
    else:
     name='liveOwnedDeepStackAndEditedBreakpointRetention';command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.DebuggerRetentionTest.'+name,'--rerun']
    log=ROOT/'target'/f'debugger-retention-{args.host}.log'
    with log.open('w') as output:result=subprocess.run(command,cwd=ROOT,env=env,stdout=output,stderr=subprocess.STDOUT)
    if result.returncode:raise RuntimeError(f'{args.host} actual acceptance failed; inspect {log}')
    if args.host=='apple':
     text=log.read_text();assert re.search(re.escape(name)+r'.*passed',text) and 'Executed 1 test, with 0 failures' in text and 'Test skipped' not in text,str(log)
    else:
     report=ROOT/'apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.DebuggerRetentionTest.xml';rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(key) is not None for key in ['skipped','failure','error']),str(report)
    assert (root/'deep.py').read_text()==SOURCE+EDIT
   print(f'PASS {args.host}: actual debugpy stack beyond 100 callers, scoped deep-frame reading, retained edited-source wishes without stale setters, explicit Save and fresh launch, retained editor/Undo, unchanged saved source during draft Undo. Physical input/accessibility acceptance remains unverified.')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()

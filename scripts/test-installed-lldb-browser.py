#!/usr/bin/env python3
"""Focused installed LLDB simple-browser launch/attach acceptance; owned temporary programs only."""
import argparse,importlib.util,json,os,re,shutil,signal,socket,subprocess,tempfile,time,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SOURCE='#include <stdio.h>\n#include <unistd.h>\nint main(int argc,char **argv) {\n volatile int value=41;\n if(argc>1) { for(;;) { if(argc>2) { FILE *out=fopen(argv[2],"a"); if(out) { fputs(".",out); fclose(out); } } usleep(10000); } }\n value+=1; /* owned-stop */\n printf("%d\\n",value);\n return 0;\n}\n'
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--host',choices=['browser','apple','android'],default='browser');parser.add_argument('--skip-build',action='store_true');parser.add_argument('--mode',choices=['launch','attach'],required=True);parser.add_argument('--engine',type=Path,default=ROOT/'target/source-edit-focus/debug/examples/engine_client');args=parser.parse_args()
 adapter=shutil.which('lldb-dap')
 if not adapter:
  found=subprocess.run(['/usr/bin/xcrun','--find','lldb-dap'],capture_output=True,text=True,check=True);adapter=found.stdout.strip()
 assert Path(adapter).is_file(),'Installed LLDB adapter required; this witness does not install one'
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 def port():
  with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));return reservation.getsockname()[1]
 def wait(read,reason):
  deadline=time.monotonic()+40
  while time.monotonic()<deadline:
   if read():return
   time.sleep(.03)
  detail=driver.act('debug-diagnostics') if driver else None
  raise AssertionError(reason+'\n'+json.dumps(detail,ensure_ascii=False))
 with tempfile.TemporaryDirectory(prefix='hickory-lldb-browser-') as temporary:
  folder=Path(temporary).resolve();root=folder/'workspace';root.mkdir();(root/'.git').mkdir();(root/'.hickory-installed-lldb-fixture').write_text('owned-installed-lldb-v1\n');(root/'main.c').write_text(SOURCE);(root/'project.md').write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources=""/></h:doc>');program=root/'program'
  subprocess.run(['/usr/bin/xcrun','clang','-g','-O0',str(root/'main.c'),'-o',str(program)],check=True,capture_output=True)
  processes=[];driver=None;target=None
  try:
   profile={'id':'owned-c','name':'Owned C '+args.mode,'program':'main.c','artifact':'program','default':True}
   if args.mode=='attach':
    heartbeat=folder/'target-heartbeat';target=fixture.start([str(program),'wait',str(heartbeat)],cwd=root);wait(lambda:heartbeat.exists(),'Owned target did not start');processes.append(target);endpoint=port();provider=fixture.start([adapter,'--connection',f'listen://127.0.0.1:{endpoint}']);processes.append(provider)
    def listening():
     # Connecting would consume this single-session DAP endpoint. Inspect owned provider readiness instead.
     result=subprocess.run(['/usr/sbin/lsof','-nP','-a','-p',str(provider.pid),'-iTCP:'+str(endpoint),'-sTCP:LISTEN'],capture_output=True,text=True);return result.returncode==0
    wait(listening,'Owned LLDB adapter did not listen');profile={'id':'owned-c','name':'Owned C attach','attach':{'port':endpoint,'arguments':{'pid':target.pid,'program':str(program),'stopOnEntry':True}},'default':True}
   (root/'.hick-launch.json').write_text(json.dumps({'version':1,'profiles':[profile]}));browser_port=port();env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')}
   engine=fixture.start([str(args.engine.resolve()),str(root),'window',f'http://127.0.0.1:{browser_port}'],env=env);processes.append(engine);origin=fixture.origin(engine)
   if args.host=='browser':vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(browser_port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite)
   if args.host=='browser':
    driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path='main.c');driver.act('remember-editor',path='main.c');geometry=driver.act('editor-size',path='main.c')
    if args.mode=='launch':driver.act('debug-breakpoint',line=6)
    driver.act('debug-start',path='main.c');wait(lambda:any('paused' in text.lower() for text in driver.act('debug-state')),'LLDB did not pause: '+str(driver.act('debug-state')))
    if args.mode=='launch':assert driver.act('debug-watch',expression='value',expected='41')=='41'
    else:assert heartbeat.exists();paused_bytes=heartbeat.stat().st_size
    assert driver.act('editor-is-retained',path='main.c') and driver.act('editor-size',path='main.c')==geometry;assert (root/'main.c').read_text()==SOURCE
    driver.act('debug-stop');wait(lambda:not any(any(word in text.lower() for word in ['paused','running','starting','unconfirmed','stopping']) for text in driver.act('debug-state')),'LLDB did not settle after Stop')
    if target:wait(lambda:heartbeat.stat().st_size>paused_bytes,'Detached target did not resume its owned heartbeat')
    assert not driver.act('debug-diagnostics')['errors'];assert (root/'main.c').read_text()==SOURCE
    print('PASS installed LLDB browser '+args.mode+': real C provider, retained editor/geometry, scoped launch value or surviving detached target, unchanged source. Physical input/accessibility unverified.')
   else:
    env.update(HICKORY_LLDB_ORIGIN=origin,HICKORY_LLDB_WORKSPACE=str(root),HICKORY_LLDB_MODE=args.mode)
    name='testLiveOwnedInstalledLLDB' if args.host=='apple' else 'liveOwnedInstalledLLDB'
    command=(['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','InstalledLLDBTests.'+name+'$']+(['--skip-build'] if args.skip_build else [])) if args.host=='apple' else [str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.InstalledLLDBTest.'+name,'--rerun']
    log=ROOT/'target'/f'installed-lldb-{args.host}-{args.mode}.log'
    with log.open('w') as output:result=subprocess.run(command,cwd=ROOT,env=env,stdout=output,stderr=subprocess.STDOUT)
    assert result.returncode==0,str(log)
    if args.host=='apple':
     text=log.read_text();assert re.search(re.escape(name)+r'.*passed',text) and 'Executed 1 test, with 0 failures' in text and 'Test skipped' not in text,str(log)
    else:
     report=ROOT/'apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.InstalledLLDBTest.xml';rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(key) is not None for key in ['skipped','failure','error']),str(report)
    if target:
     resumed_bytes=heartbeat.stat().st_size;wait(lambda:heartbeat.stat().st_size>resumed_bytes,'Detached native target did not resume its owned heartbeat')
    assert (root/'main.c').read_text()==SOURCE
    print('PASS installed LLDB '+args.host+' '+args.mode+': native controller, retained editor state, source unchanged; physical input/accessibility unverified.')
  finally:
   if driver:driver.close()
   for process in reversed(processes):
    if process is not target:fixture.stop(process)
   if target:
    try:os.killpg(target.pid,signal.SIGKILL)
    except ProcessLookupError:pass
    target.wait(timeout=5)
if __name__=='__main__':main()

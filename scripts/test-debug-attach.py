#!/usr/bin/env python3
"""Owned debugpy endpoint attach through the existing browser Debug gesture."""
import argparse, importlib.util, json, os, signal, socket, tempfile, time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def sys_exit(code):
 raise SystemExit(code)
def wait(check,message):
 deadline=time.monotonic()+30
 while time.monotonic()<deadline:
  value=check()
  if value:return value
  time.sleep(.03)
 raise AssertionError(message)
def port():
 with socket.socket() as reservation:
  reservation.bind(('127.0.0.1',0));return reservation.getsockname()[1]
def main():
 signal.signal(signal.SIGTERM,lambda signum,_frame:sys_exit(128+signum))
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--serve-only',action='store_true')
 parser.add_argument('--engine',type=Path,default=ROOT/'target/source-edit-focus/debug/examples/engine_client')
 parser.add_argument('--venv',type=Path,default=ROOT/'target/thread-debug-env')
 args=parser.parse_args();python=args.venv.resolve()/'bin/python'
 if not python.is_file():parser.error('Provide the prepared debugpy environment with --venv')
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py')
 fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 processes=[];driver=None
 with tempfile.TemporaryDirectory(prefix='hickory-debug-attach-') as folder:
  temporary=Path(folder);root=temporary/'workspace';root.mkdir();(root/'.git').mkdir()
  endpoint=port();ready=temporary/'target-ready';finish=temporary/'finish-target'
  source=('import debugpy, time\nfrom pathlib import Path\nvalue = 41\n'
   f'debugpy.listen(("127.0.0.1", {endpoint}))\n'
   f'Path({str(ready)!r}).write_text("ready")\n'
   'debugpy.wait_for_client()\ndebugpy.breakpoint()\n'
   f'while not Path({str(finish)!r}).exists():\n    time.sleep(0.02)\n')
  (root/'main.py').write_text(source)
  (root/'.hick-launch.json').write_text(json.dumps({'version':1,'profiles':[{'id':'external','name':'Owned external Python','attach':{'port':endpoint,'arguments':{'justMyCode':False}},'default':True}]}))
  environment={**os.environ,'HICKORY_STATE_DIR':str(temporary/'state')}
  try:
   target=fixture.start([str(python),str(root/'main.py')],cwd=root);processes.append(target)
   wait(lambda:ready.exists(),'Owned external debugpy endpoint did not become ready')
   web_port=port()
   server=fixture.start([str(args.engine.resolve()),str(root),'window',f'http://127.0.0.1:{web_port}'],env=environment);processes.append(server)
   origin=fixture.origin(server)
   if args.serve_only:
    print(json.dumps({'origin':origin,'workspace':str(root),'input':'main.py','endpoint':endpoint,'target_pid':target.pid,'provider':'owned debugpy endpoint','acceptance':'not run'}),flush=True)
    while True:time.sleep(1)
   vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(web_port),'--strictPort'],cwd=ROOT/'apps/web',env={**environment,'HICKORY_API_ORIGIN':origin});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html')
   driver.act('open',path='main.py');driver.act('remember-editor',path='main.py')
   size=driver.act('editor-size',path='main.py');driver.act('debug-start',path='main.py')
   wait(lambda:any('paused' in status.lower() for status in driver.act('debug-state')),'Existing Debug gesture did not attach and pause')
   assert target.poll() is None
   driver.act('source-tests-edit',path='main.py',text='\n# retained unsaved attach draft\n')
   selection=driver.act('construction-source',path='main.py',selection=True)
   assert driver.act('debug-watch',expression='value')=='41'
   driver.act('debug-step')
   wait(lambda:any('paused' in status.lower() for status in driver.act('debug-state')),'Attached process did not step')
   assert driver.act('editor-is-retained',path='main.py') and driver.act('editor-size',path='main.py')==size
   assert driver.act('construction-source',path='main.py',selection=True)==selection
   driver.act('debug-stop');time.sleep(.15)
   assert target.poll() is None,'Disconnect terminated an externally owned target'
   assert (root/'main.py').read_text()==source
   assert not driver.act('debug-diagnostics')['errors']
   driver.act('undo',path='main.py');assert driver.act('construction-source',path='main.py')==source
   finish.write_text('Allow only this owned fixture target to finish')
   target.wait(timeout=15);assert target.returncode==0
   print('PASS real debugpy endpoint/browser: existing Debug profile, attach/pause/watch/step, detach leaves external target running, retained editor/selection/geometry/undo/unsaved text and unchanged source disk',flush=True)
  except Exception:
   if driver:
    print('Attach diagnostics:',json.dumps(driver.act('debug-diagnostics'),ensure_ascii=False),flush=True)
    print('Attach wire:',json.dumps(driver.act('debug-wire')),flush=True)
   raise
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()

#!/usr/bin/env python3
"""Owned stdio-provider/browser adapter-source flow, including held IO and late receipts."""
import argparse, importlib.util, json, os, socket, sys, tempfile, time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
RUNTIME_SOURCE='<b>adapter-owned literal source</b>\nmarker = 41\n'
ADAPTER=r"""
import json, pathlib, sys, threading, time
root=pathlib.Path(sys.argv[1]); sequence=0; lock=threading.Lock(); line=2
source='<b>adapter-owned literal source</b>\nmarker = 41\n'
def emit(value):
 global sequence
 with lock:
  sequence+=1;value['seq']=sequence;body=json.dumps(value,ensure_ascii=False).encode();sys.stdout.buffer.write(('Content-Length: %d\r\n\r\n'%len(body)).encode()+body);sys.stdout.buffer.flush()
def reply(request,body=None,error=None):emit({'type':'response','request_seq':request['seq'],'command':request['command'],'success':error is None,'message':error,'body':body or {}})
def stopped(reason):emit({'type':'event','event':'stopped','body':{'reason':reason,'threadId':1,'allThreadsStopped':True}})
def held_source(request):
 (root/'source-held').write_text('provider received source')
 while not (root/'release-source').exists():time.sleep(.005)
 reply(request,{'content':source,'mimeType':'text/html'})
while True:
 headers={}
 while True:
  row=sys.stdin.buffer.readline()
  if not row:sys.exit(0)
  if row in (b'\r\n',b'\n'):break
  key,value=row.decode().split(':',1);headers[key.lower()]=value.strip()
 request=json.loads(sys.stdin.buffer.read(int(headers['content-length'])));command=request['command']
 if command=='initialize':reply(request,{'supportsConfigurationDoneRequest':True,'supportsConditionalBreakpoints':True})
 elif command=='launch':emit({'type':'event','event':'initialized','body':{}});reply(request)
 elif command=='setBreakpoints':reply(request,{'breakpoints':[{'id':i+1,'verified':True,'line':value['line']} for i,value in enumerate(request['arguments']['breakpoints'])]})
 elif command=='configurationDone':reply(request);stopped('owned adapter fixture')
 elif command=='threads':reply(request,{'threads':[{'id':1,'name':'owned-thread'}]})
 elif command=='stackTrace':reply(request,{'totalFrames':1,'stackFrames':[] if request['arguments'].get('startFrame',0) else [{'id':1,'name':'owned-runtime','line':line,'column':1,'source':{'path':str(root/'main.py'),'sourceReference':123}}]})
 elif command=='scopes':reply(request,{'scopes':[]})
 elif command=='source':
  if request['arguments'].get('sourceReference')!=123:reply(request,error='Unknown owned reference')
  elif (root/'source-error').exists():reply(request,error='Owned source provider unavailable')
  elif (root/'hold-source').exists():threading.Thread(target=held_source,args=(request,),daemon=True).start()
  else:reply(request,{'content':source,'mimeType':'text/html'})
 elif command in ('next','stepIn','stepOut','continue'):
  (root/'movement-received').write_text(command);line=3;reply(request);stopped('step')
 elif command=='evaluate':reply(request,{'result':'41','variablesReference':0,'type':'int'})
 elif command=='disconnect':reply(request);break
 else:reply(request)
"""
def prepare(root):
 root.mkdir();root=root.resolve();(root/'.git').mkdir();(root/'.hickory-owned-adapter-source-fixture').write_text('adapter-source-fixture-v1\n')
 (root/'main.py').write_text("marker = 0\nprint('workspace bytes, not adapter source')\n")
 (root/'adapter.py').write_text(ADAPTER)
 executable=root/'.venv/bin/python3';executable.parent.mkdir(parents=True)
 executable.write_text('#!'+sys.executable+'\nimport os,sys\nif "-c" in sys.argv: sys.exit(0)\nos.execv('+repr(sys.executable)+',['+repr(sys.executable)+','+repr(str(root/'adapter.py'))+','+repr(str(root))+'])\n');executable.chmod(0o755)
 return root

def wait(check,message):
 deadline=time.monotonic()+20
 while time.monotonic()<deadline:
  value=check()
  if value:return value
  time.sleep(.025)
 raise AssertionError(message)
def browser_flow(fixture,origin,root,environment,port,processes):
 vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**environment,'HICKORY_API_ORIGIN':origin});processes.append(vite)
 driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html')
 try:
  driver.act('open',path='main.py');driver.act('debug-start',path='main.py');wait(lambda:any('paused' in value.lower() for value in driver.act('debug-state')),'Owned provider did not pause')
  driver.act('source-tests-edit',path='main.py',text='\n# retained unsaved draft\n');driver.act('remember-editor',path='main.py');size=driver.act('editor-size',path='main.py');before=driver.act('construction-source',path='main.py',selection=True)
  driver.act('adapter-source-request');state=wait(lambda:(value if (value:=driver.act('adapter-source-state'))['pre']==RUNTIME_SOURCE else None),'Adapter literal source did not arrive')
  assert not state['bold'] and 'workspace bytes' not in state['pre'];assert driver.act('editor-is-retained',path='main.py');assert driver.act('editor-size',path='main.py')==size;assert driver.act('construction-source',path='main.py',selection=True)==before
  (root/'hold-source').write_text('hold optional provider response');driver.act('adapter-source-request');wait(lambda:(root/'source-held').exists(),'Optional provider request was not held')
  driver.act('debug-step');wait(lambda:(root/'movement-received').exists(),'Slow source provider blocked unrelated debugger stepping');wait(lambda:any('paused' in value.lower() for value in driver.act('debug-state')),'Step did not report its replacement stop')
  (root/'release-source').write_text('release obsolete response');(root/'hold-source').unlink();state=wait(lambda:(value if 'Historical stopped-frame receipt' in (value:=driver.act('adapter-source-state'))['text'] else None),'Old source receipt acquired the new stop')
  (root/'source-error').write_text('explicit capability refusal');driver.act('adapter-source-request');wait(lambda:'unavailable' in driver.act('adapter-source-state')['text'].lower(),'Source provider refusal was not shown')
  assert any('paused' in value.lower() for value in driver.act('debug-state'));assert driver.act('debug-watch',expression='marker')=='41'
  assert driver.act('editor-is-retained',path='main.py') and driver.act('editor-size',path='main.py')==size;assert driver.act('construction-source',path='main.py',selection=True)==before
  assert (root/'main.py').read_text()=="marker = 0\nprint('workspace bytes, not adapter source')\n"
  wire=driver.act('debug-wire');assert any(row.get('op')=='source' for row in wire);assert not driver.act('debug-diagnostics')['errors']
  driver.act('debug-stop');driver.act('undo',path='main.py');assert driver.act('construction-source',path='main.py')==(root/'main.py').read_text()
  print('PASS owned stdio adapter/browser source: exact stopped reference, literal MIME rendering, no workspace fallback, held-provider stepping, late-receipt retirement, local refusal, watch/Stop and retained editor/selection/geometry/undo/unsaved text',flush=True)
 except Exception:
  print("Owned source fixture diagnostics:",json.dumps(driver.act("debug-diagnostics"),ensure_ascii=False),flush=True)
  print("Owned source fixture wire:",json.dumps(driver.act("debug-wire")),flush=True)
  raise
 finally:driver.close()
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--serve-only',action='store_true');args=parser.parse_args()
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-adapter-source-') as folder:
  temporary=Path(folder);root=prepare(temporary/'workspace');environment={**os.environ,'HICKORY_STATE_DIR':str(temporary/'state')};processes=[]
  with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
  process=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',f'http://127.0.0.1:{port}'],env=environment);processes.append(process)
  try:
   origin=fixture.origin(process)
   if args.serve_only:
    print(json.dumps({'origin':origin,'workspace':str(root),'input':'main.py','acceptance':'not run','provider':'owned stdio fixture'}),flush=True)
    while True:time.sleep(1)
   else:browser_flow(fixture,origin,root,environment,port,processes)
  except KeyboardInterrupt:pass
  finally:
   for item in reversed(processes):fixture.stop(item)
if __name__=='__main__':main()

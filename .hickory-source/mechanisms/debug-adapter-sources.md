<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read adapter-owned source without borrowing workspace authority

For maintainers opening generated or runtime source reported only by a debugger.

A positive source reference belongs to the adapter session and exact stopped
frame. Require that frame to report the same reference before requesting bytes,
and recheck the stopped scope afterward. The session operation lock admits the already-reported reference, then releases
before provider IO. A slow optional source read cannot block stepping or Stop.
Motion retires the reference table, and a late response must recheck its exact
pause/reference before becoming a receipt. Final confirmation uses a nonblocking
operation lease: if motion or another frame operation is in flight, preserve the
old readable content and refuse this reply rather than waiting behind it or
certifying a pause during an adapter effect. Another frame, thread or pause cannot reuse its
receipt. The returned digest describes adapter-reported content; it is not proof
of a frozen executed revision and grants no filesystem edit or breakpoint right.

The Session source request bounds text to 8 MiB and MIME metadata to 256 bytes.
Hosts show literal read-only text, even if the adapter reports an HTML MIME type.
A missing provider response remains an explicit capability error; a same-named
workspace file is never substituted.

<ui:copy id="debug-adapter-source-admission">
//! Adapter source content is a stopped-frame observation, not workspace authority.
use super::{Failure, Response, observed_frame, observed_scope};
use crate::debug_sessions::Registry;
use serde::Deserialize;
use std::sync::Arc;
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub session: String,
    pub stop: u64,
    pub thread: i64,
    pub frame: i64,
    pub reference: i64,
    pub request_id: String,
}
type References = std::collections::HashMap<(u64, i64, i64), i64>;
pub(super) fn reported(frames: &[hick_dap::Frame], stop: u64, thread: i64) -> References {
    frames.iter().filter_map(|frame| frame.source_reference.filter(|reference| (1..=i32::MAX as i64).contains(reference)).map(|reference| ((stop, thread, frame.id), reference))).collect()
}
fn require_reference(references: &References, stop: u64, thread: i64, frame: i64, reference: i64) -> anyhow::Result<()> {
    anyhow::ensure!((1..=i32::MAX as i64).contains(&reference) && references.get(&(stop, thread, frame)) == Some(&reference), "This source reference was not reported in the selected stopped frame. Read its stack again.");
    Ok(())
}
pub(super) async fn read(registry: &Arc<Registry>, request: Request) -> Result<Vec<Response>, Failure> {
    let id = request.session.clone();
    let result: anyhow::Result<Vec<Response>> = async {
        let live = registry.get(&id).await?;
        {
            let _admission = live.operation.lock().await;
            observed_scope(&live, Some(request.stop), Some(request.thread), Some(&request.request_id)).await?;
            observed_frame(&live, request.stop, request.thread, Some(request.frame)).await?;
            require_reference(&*live.adapter_sources.lock().await, request.stop, request.thread, request.frame, request.reference)?;
        }
        // An optional source provider must not hold the movement gate. A late
        // response is readonly and is rejected after a changed pause/reference.
        let (content, mime_type) = live.session.source_content(request.reference).await?;
        let source_hash = super::super::workspace_fs::view::hash(content.as_bytes());
        let _confirmation = live.operation.try_lock().map_err(|_| anyhow::anyhow!("The debugger is changing or reading another frame. Read adapter source again after the current pause is confirmed."))?;
        observed_scope(&live, Some(request.stop), Some(request.thread), Some(&request.request_id)).await?;
        observed_frame(&live, request.stop, request.thread, Some(request.frame)).await?;
        require_reference(&*live.adapter_sources.lock().await, request.stop, request.thread, request.frame, request.reference)?;
        Ok(vec![Response::AdapterSource {session:id.clone(),stop:request.stop,thread:request.thread,frame:request.frame,reference:request.reference,request_id:request.request_id,content,source_hash,mime_type}])
    }.await;
    result.map_err(|error| (Some(id), error))
}
</ui:copy>
## Refuse borrowed references before adapter effects

The focused admission cases distinguish two frames' references and a same-named
workspace frame with no adapter reference. Wire checks require the whole explicit
stop/thread/frame context. Real stdio Session checks exercise literal source, MIME
and budget responses separately; live supported-provider and host acceptance
remain required before this workflow is complete.

<ui:copy id="debug-adapter-source-tests">
#[cfg(test)]
mod tests {
    use super::*;
    fn frame(id:i64,reference:Option<i64>)->hick_dap::Frame {
        serde_json::from_value(serde_json::json!({"id":id,"name":"runtime frame","line":null,"source":"same-name.py","source_line":0,"source_hash":null,"source_reference":reference,"in_document":false})).unwrap()
    }
    #[test]
    fn references_belong_to_the_reported_frame_and_never_borrow_a_workspace_name() {
        let frames=vec![frame(1,Some(123)),frame(2,Some(456)),frame(3,None)];
        let references=reported(&frames,7,9);
        assert!(require_reference(&references,7,9,1,123).is_ok());
        for (id,reference) in [(1,456),(2,123),(3,123),(4,123),(1,0),(1,-1),(1,i32::MAX as i64+1)] {assert!(require_reference(&references,7,9,id,reference).is_err());}
        assert!(require_reference(&references,8,9,1,123).is_err());
        assert!(require_reference(&references,7,10,1,123).is_err());
        assert!(require_reference(&References::new(),7,9,1,123).is_err());
    }
    #[test]
    fn wire_requires_a_complete_explicit_observation_and_rejects_unknown_fields() {
        let base=serde_json::json!({"op":"source","session":"owned","stop":1,"thread":2,"frame":3,"reference":123,"request_id":"source-request"});
        assert!(serde_json::from_value::<super::super::Request>(base.clone()).is_ok());
        for field in ["session","stop","thread","frame","reference","request_id"] {let mut value=base.clone();value.as_object_mut().unwrap().remove(field);assert!(serde_json::from_value::<super::super::Request>(value).is_err());}
        let mut value=base;value["path"]=serde_json::json!("same-name.py");assert!(serde_json::from_value::<super::super::Request>(value).is_err());
    }
}
</ui:copy>

## Drive a real browser through an owned stdio adapter

This fixture deliberately serves runtime content under the same reported filename
as different workspace bytes. It exercises the real engine, channel and floating
browser controls, then holds a provider response while stepping and releases the
obsolete reply. Provider refusal must remain local; editor identity, geometry,
selection, undo and unsaved text must remain intact. The adapter is an explicitly
owned protocol fixture, not evidence for live debugpy or another vendor adapter.
The serve-only mode supports bounded native-host acceptance on the same provider.

<ui:copy id="debug-adapter-source-real-browser-witness">
#!/usr/bin/env python3
"""Owned stdio-provider/browser adapter-source flow, including held IO and late receipts."""
import argparse, importlib.util, json, os, signal, socket, sys, tempfile, time
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
def stop_signal(signum,_frame):
 raise SystemExit(128+signum)
def main():
 signal.signal(signal.SIGTERM,stop_signal)
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
</ui:copy>
<ui:file path="scripts/test-debug-adapter-sources.py"><ui:paste select="#debug-adapter-source-real-browser-witness"/></ui:file>

<ui:file path="crates/hickory-cli/src/serve/debug_adapter_sources.rs"><ui:paste select="#debug-adapter-source-admission"/><ui:paste select="#debug-adapter-source-tests"/></ui:file>
</ui:doc>

import { BrowserRun } from "./runtime";
import type { BrowserInput,BrowserEvent } from "./runtime";
import type { DebugBreakpoint, Step } from "../client";

let run: BrowserRun | null = null;
let timer: ReturnType<typeof setTimeout> | null = null;
function schedule() {
  if (timer !== null) return;
  timer = setTimeout(() => { timer = null; if (run?.pump()) schedule(); }, 0);
}
// Validate provider-owned pause/reference state before dispatch; packet scope is
// never accepted merely because a caller echoed the right-looking numbers.
let pending:Record<string,unknown>|null=null;
function publish(event:BrowserEvent){
  if(event.event==='started')event={...event,request_id:typeof pending?.request_id==='string'?pending.request_id:undefined};
  if(pending&&['value','children','breakpoints','failed'].includes(event.event))event={...event,request_id:pending.request_id} as BrowserEvent;
  if(event.event==='stopped'&&pending?.op==='state'&&pending.frame!==undefined)event={event:'frame_variables',session:event.session,stop:event.stop!,thread:0,frame:Number(pending.frame),request_id:String(pending.request_id),variables:event.variables};
  globalThis.postMessage(event);
}
globalThis.onmessage = (event: MessageEvent<Record<string, unknown> & { input: BrowserInput }>) => {
  const request=event.data;pending=request;
  try {
    if(request.op==='start'){
      if(typeof request.request_id!=='string'||request.source_hash!==request.input.sourceHash)throw Error('Missing captured browser source proof.');
      run?.stop();run=new BrowserRun(request.input,String(request.session),String(request.doc),request.breakpoints as DebugBreakpoint[],publish);schedule();return;
    }
    if(!run||request.session!==run.session)throw Error('Unknown browser session');
    const effect=['step','eval','children'].includes(String(request.op))||request.op==='state'&&request.frame!==undefined;
    if(effect){if(typeof request.request_id!=='string')throw Error('Missing inspection request identity');run.assertScope(request.stop,request.thread,request.frame,request.op==='step'&&request.frame===undefined);}
    switch(request.op){
      case 'step':run.step(request.how as Step);schedule();break;
      case 'pause':run.pause();schedule();break;
      case 'threads':run.threads(String(request.request_id));break;
      case 'select_thread':run.assertScope(request.stop,request.thread,undefined,true);run.report();run.threads(String(request.request_id));break;
      case 'state':if(request.frame!==undefined)run.report('frame',Number(request.frame));else throw Error('No queued browser stop is available');break;
      case 'breakpoints':if(typeof request.request_id!=='string'||request.source_hash!==run.input.sourceHash)throw Error('Breakpoint source proof differs from captured browser source');run.setBreakpoints(request.breakpoints as DebugBreakpoint[]);break;
      case 'eval':if(typeof request.expression!=='string')throw Error('Expression must be a string');run.evaluate(request.expression,Number(request.frame));break;
      case 'children':if(typeof request.reference!=='number'||!Number.isSafeInteger(request.reference)||request.reference<=0)throw Error('Invalid issued value reference');run.children(request.reference,Number(request.frame));break;
      default:throw Error(`Unsupported browser debug action: ${request.op}`);
    }
  } catch(e){publish({event:'failed',session:run?.session??null,doc:request.doc,about:request.op,request_id:request.request_id,scope_retired:false,message:e instanceof Error?e.message:String(e)} as BrowserEvent);}
  finally{pending=null;}
};

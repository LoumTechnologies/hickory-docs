<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read adapter source inside the existing floating debugger

For maintainers showing runtime-only source without retargeting a workspace editor.

## Bind each read to the exact observed pause, frame and source reference

Retain a one-shot request before sending. A reply must echo every identity, match
its UTF-8 content digest and still belong to the captured editor/session scope.
Retire pending reads on motion or selection changes. Retained text may remain
readable as a historical receipt, but cannot authorize precise filesystem
navigation, editing or breakpoint changes.

<ui:copy id="debug-adapter-source-browser-model">
export interface AdapterSourceScope {session:string;stop:number;thread:number;frame:number;reference:number}
export interface AdapterSourceEvent extends AdapterSourceScope {event:'adapter_source';request_id:string;content:string;source_hash:string;mime_type:string|null}
interface PendingSource {scope:AdapterSourceScope;request:string;current:()=>boolean}
export class AdapterSourceReading {
 value:AdapterSourceEvent|null=null;notice='';busy=false;private pending:PendingSource|null=null;private heldCurrent:()=>boolean=()=>false;
 constructor(private changed:()=>void=()=>{}){}
 close(){this.pending=null;this.busy=false;this.value=null;this.notice='';this.changed();}
 retire(){if(this.pending){this.pending=null;this.busy=false;this.notice='Adapter source request retired with its stopped context.';this.changed();}}
 current(){return this.value!==null&&this.heldCurrent();}
 read(scope:AdapterSourceScope,current:()=>boolean,send:(scope:AdapterSourceScope,request:string)=>void){
  if(!scope.session||!Number.isSafeInteger(scope.stop)||scope.stop<0||![scope.thread,scope.frame].every(value=>Number.isInteger(value)&&value>=-2147483648&&value<=2147483647)||!Number.isInteger(scope.reference)||scope.reference<=0||scope.reference>2147483647||!current()){this.notice='Select a current stopped frame with adapter-owned source.';this.changed();return;}
  const pending={scope:{...scope},request:crypto.randomUUID(),current};this.pending=pending;this.busy=true;this.notice='Reading literal adapter-owned source…';this.changed();
  try{send(pending.scope,pending.request);}catch(error){if(this.pending===pending){this.pending=null;this.busy=false;this.notice=String(error);this.changed();}}
 }
 async accept(input:AdapterSourceEvent){const event={...input};const pending=this.pending;if(!pending||event.request_id!==pending.request||!(['session','stop','thread','frame','reference'] as const).every(key=>event[key]===pending.scope[key]))return;
  try{if(typeof event.content!=='string'||new TextEncoder().encode(event.content).length>8*1024*1024||!/^[a-f0-9]{64}$/.test(event.source_hash)||event.mime_type!==null&&(typeof event.mime_type!=='string'||new TextEncoder().encode(event.mime_type).length>256||/[\u0000-\u001f\u007f]/.test(event.mime_type)))throw Error('Adapter source receipt is malformed or exceeds its reading budget.');
   const hash=[...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(event.content)))].map(byte=>byte.toString(16).padStart(2,'0')).join('');
   if(this.pending!==pending)return;if(!pending.current())throw Error('Stopped frame changed before adapter source arrived.');if(hash!==event.source_hash)throw Error('Adapter source content fingerprint changed.');this.value=Object.freeze({...event});this.heldCurrent=pending.current;this.notice='Adapter-reported content only. No workspace source, edit or breakpoint authority is implied.';
  }catch(error){if(this.pending===pending)this.notice=String(error);}finally{if(this.pending===pending){this.pending=null;this.busy=false;this.changed();}}
 }
 failed(session:string|null|undefined,request:string|null|undefined,message:string){if(!this.pending||this.pending.scope.session!==session||this.pending.request!==request)return false;this.pending=null;this.busy=false;this.notice=message;this.changed();return true;}
}
</ui:copy>

## Render literal content without trusting the reported MIME type

The existing debugger popover contains this source reading. Page text without splitting Unicode scalars or changing the code editor or its scroll/selection. Even text/html is shown
literally, and a reported filename never triggers a same-named workspace fallback.

<ui:copy id="debug-adapter-source-browser-view">
import {useState} from 'react';
import type {AdapterSourceReading} from './AdapterSourceReading';
export function AdapterSourcePanel({model}:{model:AdapterSourceReading}){
 const [page,setPage]=useState(0),value=model.value,length=value?.content.length??0;
 const crosses=(at:number)=>!!value&&at>0&&/[\uD800-\uDBFF]/.test(value.content[at-1]??'')&&/[\uDC00-\uDFFF]/.test(value.content[at]??'');
 const last=Math.max(0,Math.ceil(length/4000)-1-(length>4000&&length%4000===1&&crosses(length-1)?1:0)),shown=Math.min(page,last);
 const start=shown*4000,end=(shown+1)*4000,from=start+(crosses(start)?1:0),to=Math.min(length,end+(crosses(end)?1:0));
 if(!value&&!model.notice&&!model.busy)return null;
 return <section aria-label="Adapter-owned source"><header><strong>Adapter-owned source</strong><button onClick={()=>model.close()}>Close source reading</button></header><p role="status">{model.notice}</p>{value&&<><p>{model.current()?'Captured current stopped-frame receipt':'Historical stopped-frame receipt'} · reference {value.reference} · frame {value.frame} · SHA-256 {value.source_hash}</p><p>MIME observation: {value.mime_type??'unspecified'}. Literal read-only text; no workspace file was substituted.</p><pre style={{whiteSpace:'pre-wrap'}}>{value.content.slice(from,to)}</pre><button disabled={!shown} onClick={()=>setPage(shown-1)}>Previous source text</button><span>{shown+1}/{last+1}</span><button disabled={shown===last} onClick={()=>setPage(shown+1)}>Next source text</button></>}</section>;
}
</ui:copy>
## Check receipt identity, retirement, content integrity and literal rendering

Focused models reject foreign replies, stale contexts, invalid references,
oversized content and malformed MIME observations. The rendered component keeps
HTML literal and assigns a surrogate pair crossing a page boundary to exactly
one page. Concatenating consecutive pages reconstructs the exact receipt without
a duplicate scalar or an empty trailing page. These checks do
not establish live provider or cross-platform host acceptance.

<ui:copy id="debug-adapter-source-browser-tests">
import {beforeAll,expect,it,vi} from 'vitest';
import {AdapterSourceReading,type AdapterSourceEvent,type AdapterSourceScope} from './AdapterSourceReading';
beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});});
const scope:AdapterSourceScope={session:'owned',stop:2,thread:3,frame:4,reference:123};
async function event(request:string,content='literal 😀\n<b>source</b>'):Promise<AdapterSourceEvent>{return {...scope,event:'adapter_source',request_id:request,content,source_hash:[...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(content)))].map(byte=>byte.toString(16).padStart(2,'0')).join(''),mime_type:'text/html'};}
it('accepts exact reported content as a read-only receipt without filesystem fallback',async()=>{const model=new AdapterSourceReading();let id='';const send=vi.fn((_scope,request)=>{id=request;});model.read(scope,()=>true,send);expect(send).toHaveBeenCalledOnce();await model.accept(await event(id));expect(model.value?.content).toBe('literal 😀\n<b>source</b>');expect(model.current()).toBe(true);expect(model.notice).toContain('No workspace source');});
it('ignores foreign identities and rejects changed content hashes',async()=>{for(const field of ['session','stop','thread','frame','reference','request_id'] as const){const model=new AdapterSourceReading();let id='';model.read(scope,()=>true,(_scope,request)=>{id=request;});const value=await event(id);(value as unknown as Record<string,unknown>)[field]=typeof value[field]==='number'?999:'foreign';await model.accept(value);expect(model.value).toBeNull();expect(model.busy).toBe(true);}const model=new AdapterSourceReading();let id='';model.read(scope,()=>true,(_scope,request)=>{id=request;});const value=await event(id);value.content='tampered';await model.accept(value);expect(model.value).toBeNull();expect(model.busy).toBe(false);expect(model.notice).toContain('fingerprint');});
it('refuses stale/closed scopes and retires old requests before admitting a newer reference',async()=>{let current=true,id='';const model=new AdapterSourceReading();model.read(scope,()=>current,(_scope,request)=>{id=request;});const old=await event(id);current=false;await model.accept(old);expect(model.value).toBeNull();current=true;model.read(scope,()=>current,(_scope,request)=>{id=request;});const closed=await event(id);model.close();await model.accept(closed);expect(model.value).toBeNull();model.read(scope,()=>current,(_scope,request)=>{id=request;});await model.accept(await event(id));current=false;model.retire();expect(model.value?.content).toContain('literal');expect(model.current()).toBe(false);});
it('refuses invalid reference/budget/MIME responses while leaving the selected code alone',async()=>{const send=vi.fn(),model=new AdapterSourceReading();for(const reference of [0,-1,2147483648])model.read({...scope,reference},()=>true,send);expect(send).not.toHaveBeenCalled();for(const kind of ['large','mime']){let id='';model.read(scope,()=>true,(_scope,request)=>{id=request;});const value=await event(id);if(kind==='large')value.content='x'.repeat(8*1024*1024+1);else value.mime_type='text/\nhtml';await model.accept(value);expect(model.value).toBeNull();expect(model.busy).toBe(false);}});
it('correlates capability errors without retiring unrelated debugger state',()=>{const model=new AdapterSourceReading();let id='';model.read(scope,()=>true,(_scope,request)=>{id=request;});expect(model.failed('foreign',id,'no')).toBe(false);expect(model.busy).toBe(true);expect(model.failed(scope.session,id,'Provider source unavailable')).toBe(true);expect(model.notice).toContain('unavailable');expect(model.busy).toBe(false);});
</ui:copy>
<ui:copy id="debug-adapter-source-browser-view-tests">
import {beforeAll,expect,it} from 'vitest';
import {render,screen,fireEvent} from '@testing-library/react';
import {AdapterSourceReading} from './AdapterSourceReading';
import {AdapterSourcePanel} from './AdapterSourcePanel';
beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});});
it('renders MIME-labelled markup literally and pages surrogate boundaries without omitting text',()=>{const model=new AdapterSourceReading(),text='x'.repeat(3999)+'😀<b>literal</b>';model.value={event:'adapter_source',session:'owned',stop:1,thread:2,frame:3,reference:4,request_id:'request',content:text,source_hash:'a'.repeat(64),mime_type:'text/html'};const {container,unmount}=render(<AdapterSourcePanel model={model}/>);expect(container.querySelector('b')).toBeNull();expect(container.querySelector('pre')?.textContent).toContain('😀');const first=container.querySelector('pre')!.textContent!;fireEvent.click(screen.getByRole('button',{name:'Next source text'}));const second=container.querySelector('pre')!.textContent!;expect(second).toBe('<b>literal</b>');expect(first+second).toBe(text);expect(container.querySelector('b')).toBeNull();expect(screen.getByText(/Historical stopped-frame receipt/)).toBeTruthy();unmount();});
it('does not offer an empty page when the final scalar crosses the nominal boundary',()=>{const model=new AdapterSourceReading(),text='x'.repeat(3999)+'😀';model.value={event:'adapter_source',session:'owned',stop:1,thread:2,frame:3,reference:4,request_id:'request',content:text,source_hash:'a'.repeat(64),mime_type:null};const {container,unmount}=render(<AdapterSourcePanel model={model}/>);expect(container.querySelector('pre')?.textContent).toBe(text);expect((screen.getByRole('button',{name:'Next source text'}) as HTMLButtonElement).disabled).toBe(true);unmount();});
</ui:copy>
<ui:file path="apps/web/src/debug/AdapterSourceReading.test.ts"><ui:paste select="#debug-adapter-source-browser-tests"/></ui:file>
<ui:file path="apps/web/src/debug/AdapterSourcePanel.test.tsx"><ui:paste select="#debug-adapter-source-browser-view-tests"/></ui:file>

<ui:file path="apps/web/src/debug/AdapterSourceReading.ts"><ui:paste select="#debug-adapter-source-browser-model"/></ui:file>
<ui:file path="apps/web/src/debug/AdapterSourcePanel.tsx"><ui:paste select="#debug-adapter-source-browser-view"/></ui:file>
</ui:doc>

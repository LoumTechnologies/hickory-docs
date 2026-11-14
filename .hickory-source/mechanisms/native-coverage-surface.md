<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Embed structured coverage without owning the editor

For maintainers replacing optional native tool presentation.

The local shared renderer delegates service calls, run selection and source navigation to a native host. It has no network access. The host must accept only its fixed API operations and independently check workspace, source hashes and editor positions. This module alone grants no execution or write authority. Closing or changing workspace retires pending requests; numeric identities never recycle. A render failure remains inside this optional surface.

## native-coverage-bridge

Keep local rendering and transport independently replaceable. Requests are bounded and source navigation remains a host decision. The bundled page forbids remote resources, nested frames and direct network calls.

<ui:copy id="native-coverage-bridge">
import {createRoot} from "react-dom/client";
import {Component,type ErrorInfo,type ReactNode} from "react";
import {CoveragePanel} from "./CoverageTools";
import type {CoverageSurfaceClient} from "./CoverageSurface";
export interface NativeCoverageContext {workspace:string;path:string|null;language:string;selectedId:string|null}
type Post=(message:Record<string,unknown>)=>void;
/** Optional rendering stays isolated from readable native source and editor state. */
class SurfaceBoundary extends Component<{children:ReactNode;close:()=>void},{error:string|null}> {
  state={error:null as string|null};static getDerivedStateFromError(error:Error){return {error:error.message};}
  componentDidCatch(_error:Error,_info:ErrorInfo){}
  render(){return this.state.error?<section role="alert"><p>Tests and coverage could not render. Close and reopen after checking the workspace connection.</p><button onClick={this.props.close}>Close</button></section>:this.props.children;}
}
export function mountNativeCoverage(host:HTMLElement,post:Post) {
  const root=createRoot(host);let sequence=0,closed=false,context:NativeCoverageContext|null=null;
  const pending=new Map<number,{resolve:(value:unknown)=>void;reject:(error:Error)=>void;timer:ReturnType<typeof setTimeout>}>();
  const send=(message:Record<string,unknown>)=>{if(closed)throw new Error("Tool closed. Reopen Tests and coverage.");post({...message,workspace:context?.workspace??null});};
  function request<T>(kind:string,payload:Record<string,unknown>):Promise<T>{
    if(!context)return Promise.reject(new Error("Workspace not ready. Reopen Tests and coverage."));
    if(pending.size>=64)return Promise.reject(new Error("Too many pending tool requests. Wait for the workspace and retry."));
    return new Promise<T>((resolve,reject)=>{const id=++sequence;
      const timer=setTimeout(()=>{pending.delete(id);reject(new Error("Workspace request timed out. Check its state before retrying an operation."));},45_000);
      pending.set(id,{resolve:value=>resolve(value as T),reject,timer});
      try {send({kind,id,...payload});}catch(error){clearTimeout(timer);pending.delete(id);reject(error);}
    });
  }
  const close=()=>send({kind:"close"});
  const client:CoverageSurfaceClient={call:body=>request("api",{body}),select:run=>send({kind:"select",id:run?.id??null}),navigate:location=>request<void>("navigate",{location})};
  const retire=()=>{for(const value of pending.values()){clearTimeout(value.timer);value.reject(new Error("Tool context changed. Reopen the surface before retrying."));}pending.clear();};
  const surface={
    update(value:NativeCoverageContext){
      if(closed)return;
      if(context&&context.workspace!==value.workspace)retire();context=value;
      root.render(<SurfaceBoundary key={value.workspace} close={close}><CoveragePanel client={client} initialPath={value.path??"."} initialLanguage={value.language} initialSelectedId={value.selectedId} close={close} embedded/></SurfaceBoundary>);
    },
    reply(id:number,value:unknown,error:string|null){const request=pending.get(id);if(!request)return;pending.delete(id);clearTimeout(request.timer);if(error)request.reject(new Error(error));else request.resolve(value);},
    dispose(){if(closed)return;closed=true;retire();root.unmount();},
  };
  queueMicrotask(()=>{if(!closed)post({kind:"ready"});});return surface;
}
</ui:copy>

<ui:file path="apps/web/src/coverage/NativeCoverage.tsx"><ui:paste select="#native-coverage-bridge"/></ui:file>

## native-coverage-bridge-witness

The focused witness checks reply routing, context retirement, pending-request bounds, timeouts and disposal. It mocks presentation only; native host admission remains a separate responsibility.

<ui:copy id="native-coverage-bridge-witness">
import {act} from '@testing-library/react';
import {afterEach,expect,it,vi} from 'vitest';
import type {CoverageSurfaceClient} from './CoverageSurface';
const captured=vi.hoisted(()=>({client:null as CoverageSurfaceClient|null}));
vi.mock('./CoverageTools',()=>({CoveragePanel:({client}:{client:CoverageSurfaceClient})=>{captured.client=client;return <p>Coverage surface</p>;}}));
import {mountNativeCoverage} from './NativeCoverage';
afterEach(()=>{document.body.replaceChildren();captured.client=null;vi.useRealTimers();});
function fixture(){const host=document.createElement('div');document.body.append(host);const post=vi.fn();const surface=mountNativeCoverage(host,post);act(()=>surface.update({workspace:'one',path:'app.py',language:'python',selectedId:null}));return {surface,post,client:captured.client!};}
it('routes service and navigation replies by request identity',async()=>{
 const {surface,post,client}=fixture();const response=client.call({action:'list'});const message=post.mock.calls.at(-1)![0];expect(message).toMatchObject({kind:'api',workspace:'one',body:{action:'list'}});
 surface.reply(message.id+1,{wrong:true},null);surface.reply(message.id,{runs:[]},null);await expect(response).resolves.toEqual({runs:[]});
 const navigation=client.navigate({path:'app.py',line:1,sourceHash:'a'.repeat(64),sourceState:'current',runId:null});const next=post.mock.calls.at(-1)![0];expect(next.kind).toBe('navigate');surface.reply(next.id,null,'Source changed');await expect(navigation).rejects.toThrow('Source changed');act(()=>surface.dispose());
});
it('context replacement rejects pending requests and cannot reuse old replies',async()=>{
 const {surface,post,client}=fixture();const old=client.call({action:'get',id:'old'});const rejected=expect(old).rejects.toThrow('context changed');const first=post.mock.calls.at(-1)![0];
 act(()=>surface.update({workspace:'two',path:null,language:'rust',selectedId:null}));await rejected;surface.reply(first.id,{old:true},null);
 const fresh=captured.client!.call({action:'list'});const second=post.mock.calls.at(-1)![0];expect(second.workspace).toBe('two');expect(second.id).toBeGreaterThan(first.id);surface.reply(second.id,{runs:[]},null);await expect(fresh).resolves.toEqual({runs:[]});act(()=>surface.dispose());
});
it('bounds pending requests and retires them on timeout and disposal',async()=>{
 vi.useFakeTimers();const {surface,client}=fixture();const pending=Array.from({length:64},()=>client.call({action:'list'}).catch(error=>String(error)));
 await expect(client.call({action:'list'})).rejects.toThrow('Too many');await act(async()=>{vi.advanceTimersByTime(45000);});expect((await Promise.all(pending)).every(value=>typeof value==='string'&&value.includes('timed out'))).toBe(true);
 const held=client.call({action:'list'});const rejected=expect(held).rejects.toThrow('context changed');act(()=>surface.dispose());await rejected;await expect(client.call({action:'list'})).rejects.toThrow('closed');
});
</ui:copy>

<ui:file path="apps/web/src/coverage/NativeCoverage.test.tsx"><ui:paste select="#native-coverage-bridge-witness"/></ui:file>

## native-coverage-local-page

Keep local rendering and transport independently replaceable. Requests are bounded and source navigation remains a host decision. The bundled page forbids remote resources, nested frames and direct network calls.

<ui:copy id="native-coverage-local-page">
<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'none'; img-src 'none'; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'">
<title>Tests and coverage</title><link rel="stylesheet" href="node_modules/coverage.css">
<style>:root{color-scheme:light dark;--bg:Canvas;--fg:CanvasText;--border:GrayText;--accent:LinkText}body{margin:0;color:CanvasText;background:Canvas;font:13px system-ui}button{font:inherit;cursor:pointer}button:disabled{cursor:default}button.btn{padding:4px 8px;border:1px solid GrayText;border-radius:4px;color:CanvasText;background:Canvas}.error{color:MarkText;background:Mark;padding:6px}#surface>section{padding:12px}</style>
</head><body><main id="surface">Opening tests and coverage…</main><script src="node_modules/coverage.js"></script></body></html>
</ui:copy>

<ui:file path="apps/apple-simple/Sources/Resources/coverage.html"><ui:paste select="#native-coverage-local-page"/></ui:file>

</ui:doc>

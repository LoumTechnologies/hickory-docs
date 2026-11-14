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

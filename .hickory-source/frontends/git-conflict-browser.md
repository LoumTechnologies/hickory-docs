<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Inspect immutable Git conflict stages

For maintainers changing workspace services independently of the editor renderer.

Conflict inspection is an optional capability inside the existing repository review. It validates literal source identity, protocol version, immutable object names, stage uniqueness and explicit text availability. A fixed read ticket retires held/closed/foreign replies. Stage labels preserve Git’s meaning during rebase. Source text and index remain separate from this observation; this view does not publish a resolution. Page the retained stage text without reserving code space, and disclose a cap for a very long page.

<ui:copy id="git-conflict-browser">
import {useEffect,useMemo,useReducer,useState} from 'react';
import {request} from '../api/client';
export interface ConflictStage {stage:number;mode:string;oid:string;text:string|null;unavailable:string|null}
export interface ConflictReading {version:1;path:string;fingerprint:string;stages:ConflictStage[];index_modified:false;source_files_modified:false}
export function validateConflict(value:ConflictReading,path:string):void {
 if(value.version!==1||value.path!==path||value.index_modified!==false||value.source_files_modified!==false||!/^[a-f0-9]{64}$/i.test(value.fingerprint)||!Array.isArray(value.stages)||value.stages.length<1||value.stages.length>3)throw new Error('Conflict identity changed or unsupported response. Refresh the conflict.');
 const seen=new Set<number>();for(const stage of value.stages){if(![1,2,3].includes(stage.stage)||seen.has(stage.stage)||!/^(?:[a-f0-9]{40}|[a-f0-9]{64})$/i.test(stage.oid)||!/^[0-7]{6}$/.test(stage.mode)||!(typeof stage.text==='string'&&stage.unavailable===null&&['100644','100755'].includes(stage.mode)||stage.text===null&&typeof stage.unavailable==='string'))throw new Error('Malformed conflict stage. Refresh repository changes.');seen.add(stage.stage);}
}
export class GitConflictModel {
 value:ConflictReading|null=null;notice='';loading=false;private ticket=0;
 constructor(private changed:()=>void=()=>{}){}
 close(){this.ticket++;this.loading=false;this.value=null;}
 async read(path:string,load:(path:string)=>Promise<ConflictReading>){const id=++this.ticket;this.value=null;this.loading=true;this.notice='Reading unmerged stages…';this.changed();
 try{const value=await load(path);if(id!==this.ticket)return;validateConflict(value,path);this.value=value;this.notice='Captured immutable Git stages. Working text and unsaved drafts are separate.';}
 catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}}
}
export function GitConflictView({path}:{path:string}){
 const [,paint]=useReducer(n=>n+1,0),model=useMemo(()=>new GitConflictModel(paint),[path]);const [selected,setSelected]=useState(2),[page,setPage]=useState(0);
 useEffect(()=>()=>model.close(),[model]);
 const stage=model.value?.stages.find(item=>item.stage===selected)??model.value?.stages[0],rows=stage?.text?.split('\n')??[],last=Math.max(0,Math.ceil(rows.length/100)-1),index=Math.min(page,last),text=rows.slice(index*100,(index+1)*100).join('\n');
 return <section aria-label="Conflict stages"><button disabled={model.loading} onClick={()=>{setPage(0);void model.read(path,path=>request('GET','/api/git/conflict?path='+encodeURIComponent(path)));}}>Inspect conflict stages</button>{model.notice&&<p role="status">{model.notice}</p>}{model.value&&<>
 <p>Stage 1: base · Stage 2 and Stage 3 retain Git’s meanings, including during rebase.</p>
 {model.value.stages.map(item=><button key={item.stage} aria-pressed={stage===item} onClick={()=>{setSelected(item.stage);setPage(0);}}>Stage {item.stage}{item.stage===1?' · Base':''}</button>)}
 {stage&&<><p>Stage {stage.stage} · mode {stage.mode}</p><code>{stage.oid}</code>{stage.text===null?<p>{stage.unavailable}</p>:<><p>Lines {index*100+1}–{Math.min(rows.length,(index+1)*100)}{text.length>200000?' · page preview capped at 200,000 characters':''}</p><pre aria-label="Conflict stage text" style={{maxHeight:'30vh',overflow:'auto'}}>{text.slice(0,200000)}</pre><button disabled={!index} onClick={()=>setPage(index-1)}>Previous stage lines</button><button disabled={index===last} onClick={()=>setPage(index+1)}>Next stage lines</button></>}</>}
 </>}</section>;
}
</ui:copy>

<ui:file path="apps/web/src/simple/GitConflict.tsx"><ui:paste select="#git-conflict-browser"/></ui:file>
## Retire stale conflict observations

Release held conflict reads out of order, close during a pending read, and supply a byte-distinct foreign path. Only the latest exact identity may display. Validate immutable object names, unique stages and explicit binary/text availability. These are service-model witnesses, not native physical input acceptance.

<ui:copy id="git-conflict-browser-tests">
import {it,expect} from 'vitest';
import {GitConflictModel,validateConflict,type ConflictReading} from './GitConflict';
const reading=(path:string,text='captured'):ConflictReading=>({version:1,path,fingerprint:'a'.repeat(64),stages:[{stage:2,mode:'100644',oid:'b'.repeat(40),text,unavailable:null}],index_modified:false,source_files_modified:false});
it('retires older, closed and foreign conflict replies',async()=>{
 const held:((value:ConflictReading)=>void)[]=[];const load=()=>new Promise<ConflictReading>(resolve=>held.push(resolve));const model=new GitConflictModel();
 const first=model.read('a.txt',load),second=model.read('b.txt',load);held[1](reading('b.txt','new'));await second;held[0](reading('a.txt','old'));await first;expect(model.value?.path).toBe('b.txt');
 const closed=model.read('a.txt',load);model.close();held[2](reading('a.txt'));await closed;expect(model.value).toBeNull();
 await model.read('é.txt',async()=>reading('e\u0301.txt'));expect(model.value).toBeNull();expect(model.notice).toContain('identity changed');
});
it('requires complete immutable identities and unambiguous text availability',()=>{
 const value=reading('a.txt');expect(()=>validateConflict(value,'a.txt')).not.toThrow();
 for(const malformed of [{...value,fingerprint:'a'},{...value,index_modified:true},{...value,stages:[value.stages[0],value.stages[0]]},{...value,stages:[{...value.stages[0],oid:'main'}]},{...value,stages:[{...value.stages[0],mode:'120000'}]},{...value,stages:[{...value.stages[0],text:null}]}])expect(()=>validateConflict(malformed as ConflictReading,'a.txt')).toThrow();
 const binary=reading('a.txt');binary.stages[0]={...binary.stages[0],text:null,unavailable:'Binary'};expect(()=>validateConflict(binary,'a.txt')).not.toThrow();
});
</ui:copy>

<ui:file path="apps/web/src/simple/GitConflict.test.ts"><ui:paste select="#git-conflict-browser-tests"/></ui:file>
## Read actual conflict stages beside a retained draft

Use a real merge conflict and the existing repository dialog, then compare all three immutable stage texts. Keep a dirty source open, exercise its undo/redo, and assert unchanged code dimensions, editor identity, HEAD, index and saved source. The fixture and all Git writes remain inside its temporary repository.

<ui:copy id="git-conflict-real-browser">
"""Read immutable Git conflict stages while retaining an unrelated unsaved editor draft."""
import importlib.util
import os
from pathlib import Path
import socket
import subprocess
import tempfile
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location("fixture",ROOT/"scripts/test-simple-browser.py");fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix="simple-conflict-") as temporary:
  folder=Path(temporary);root=folder/"workspace";root.mkdir();path="literal[ab] name.txt"
  def git(*args,check=True):return subprocess.run(["git","-C",str(root),"--literal-pathspecs",*args],check=check,stdout=subprocess.PIPE,stderr=subprocess.PIPE).stdout
  git("init","-b","main");git("config","user.email","test@example.invalid");git("config","user.name","Conflict fixture")
  (root/path).write_text("base\n");git("add","--",path);git("commit","-m","base");git("checkout","-b","incoming")
  (root/path).write_text("incoming\n");git("commit","-am","incoming");git("checkout","main");(root/path).write_text("current\n");git("commit","-am","current");git("merge","incoming",check=False)
  saved=(root/path).read_bytes();draft=saved.decode()+"# unsaved\n";index=(root/".git/index").read_bytes();head=git("rev-parse","HEAD")
  environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
   ui=f"http://127.0.0.1:{port}"
   engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
   vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
   driver.act("construction-set-source",path=path,source=draft);driver.act("review-open");driver.act("review-select",path=path,staged=False)
   text=driver.act("review-conflict-read");assert "Captured immutable Git stages" in text and "rebase" in text,text
   for stage,expected in [(1,"base\n"),(2,"current\n"),(3,"incoming\n")]:assert driver.act("review-conflict-stage",stage=stage)==expected
   driver.act("review-close");assert driver.act("construction-source",path=path)==draft
   assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size
   driver.act("construction-history",redo=False);assert driver.act("construction-source",path=path)==saved.decode();driver.act("construction-history",redo=True);assert driver.act("construction-source",path=path)==draft
   assert (root/path).read_bytes()==saved;assert (root/".git/index").read_bytes()==index;assert git("rev-parse","HEAD")==head
   print("PASS: real conflict stage 1/2/3 inspection through repository review, literal path, retained dirty source/editor/dimensions/undo, unchanged HEAD/index/working bytes")
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
</ui:copy>

<ui:file path="scripts/test-simple-conflict-browser.py"><ui:paste select="#git-conflict-real-browser"/></ui:file>
</ui:doc>

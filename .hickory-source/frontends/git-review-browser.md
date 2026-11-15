<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read saved repository changes through existing menus

For maintainers changing workspace services independently of the editor renderer.

Review saved Git state independently of editor drafts. A small model owns list/diff requests and a monotonic intent: refreshing, changing sides or closing retires older replies. The UI only presents that state in the existing floating surface; no source, save, staging or history command runs during inspection. Keep staged and working-tree columns distinct and bound displayed diff text without presenting a truncated view as complete.

<ui:copy id="simple-git-review-browser">
import {useEffect,useMemo,useReducer,useRef} from 'react';
import {createPortal} from 'react-dom';
import {api} from '../api/client';
import type {GitChanges,GitDiff,GitChangeFile} from '../api/git';
export interface GitReviewReader {changes:()=>Promise<GitChanges>;diff:(path:string,staged:boolean)=>Promise<GitDiff>}
export class GitReviewModel {
 files:GitChangeFile[]=[];branch='';selected:string|null=null;staged=false;detail='';notice='';loading=false;page=0;
 private ticket=0;
 constructor(private readonly reader:GitReviewReader,private readonly changed:()=>void=()=>{}){}
 close(){this.ticket++;this.loading=false;}
 async refresh(){const id=++this.ticket;this.loading=true;this.files=[];this.branch='';this.selected=null;this.detail='';this.notice='Reading saved Git state…';this.changed();
  try{const value=await this.reader.changes();if(id!==this.ticket)return;this.files=value.files;this.branch=value.branch??'';this.page=0;this.notice=value.repository?'Saved Git state · unsaved editor drafts are not included.':'This workspace is not a Git repository.';}
  catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}
 }
 async select(path:string,staged:boolean){if(!this.files.some(file=>file.path===path))return;const id=++this.ticket;this.selected=path;this.staged=staged;this.detail='';this.loading=true;this.notice='Reading saved diff…';this.changed();
  try{const value=await this.reader.diff(path,staged);if(id!==this.ticket)return;if(value.path!==path)throw new Error('Diff identity changed. Refresh repository changes.');this.detail=value.diff.slice(0,200000);this.notice=value.binary?'Binary change · text comparison unavailable.':value.diff.length>200000?'First 200,000 characters shown. Use Terminal to read the complete diff.':value.diff?'Captured Git diff · refresh after external changes.':'No changes on this side.';}
  catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}
 }
}
</ui:copy>

## Present the captured reading without moving code

Use the existing floating surface and menu vocabulary. Paging limits the visible file list; staged and working-tree buttons select an explicit side. Keep saved Git state visibly separate from unsaved editor drafts, preserve code focus on close and retire pending requests when the surface disappears. Diff text stays selectable and never becomes an editor mutation.

<ui:copy id="simple-git-review-browser-view">
export function GitReviewView({visible,workspace,close}:{visible:boolean;workspace:string;close:()=>void}) {
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new GitReviewModel({changes:()=>api.gitChanges(),diff:(path,staged)=>api.gitDiff(path,staged)},paint),[workspace]);
 const panel=useRef<HTMLElement>(null);
 useEffect(()=>{if(!visible)return;void model.refresh();panel.current?.focus();return()=>model.close();},[visible,model]);
 if(!visible)return null;
 return createPortal(<section ref={panel} tabIndex={-1} className="simple-workspace-problems simple-source-tests" role="dialog" aria-label="Repository changes" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
  <header><strong>Repository changes {model.branch}</strong><button onClick={()=>void model.refresh()}>Refresh</button><button onClick={close}>Close</button></header>
  <p role="status">{model.notice}</p><p>Saved working tree and index. Unsaved editor drafts are not included.</p>
  <div aria-label="Changed files">{model.files.slice(model.page*20,(model.page+1)*20).map(file=><div key={file.path}><span>{file.from?`${file.from} → `:''}{file.path}</span> <button aria-label={`Working tree ${file.path}`} onClick={()=>void model.select(file.path,false)}>Working tree {file.tree}</button> <button aria-label={`Staged ${file.path}`} onClick={()=>void model.select(file.path,true)}>Staged {file.index}</button></div>)}</div>
  <nav aria-label="Changed file pages"><button disabled={model.page===0} onClick={()=>{model.page--;paint();}}>Previous</button><span>{model.files.length} changed files · page {model.page+1}</span><button disabled={(model.page+1)*20>=model.files.length} onClick={()=>{model.page++;paint();}}>Next</button></nav>
  {model.selected&&<><h3>{model.selected} · {model.staged?'Staged':'Working tree'}</h3><pre aria-label="Git diff" style={{maxHeight:'40vh',overflow:'auto',whiteSpace:'pre'}}>{model.detail}</pre></>}
 </section>,document.body);
}
</ui:copy>

<ui:file path="apps/web/src/simple/GitReview.tsx"><ui:paste select="#simple-git-review-browser"/><ui:paste select="#simple-git-review-browser-view"/></ui:file>
## Reject obsolete and foreign diff replies

Hold two diff replies and release them in reverse order. The newer path and side must remain selected; closing retires a held reply. A mismatched response identity refuses before displaying its text. These are service safety witnesses, independent of popover layout and native physical input.

<ui:copy id="simple-git-review-browser-tests">
import {it,expect} from 'vitest';
import {GitReviewModel} from './GitReview';
import type {GitDiff} from '../api/git';
it('retains only the latest saved diff identity and retires closed work',async()=>{
 const pending:((value:GitDiff)=>void)[]=[];
 const model=new GitReviewModel({changes:async()=>({repository:true,branch:'main',files:['a.py','b.py'].map(path=>({path,index:'M',tree:'M'}))}),diff:()=>new Promise(resolve=>pending.push(resolve))});
 await model.refresh();const first=model.select('a.py',false),second=model.select('b.py',true);
 pending[1]({path:'b.py',diff:'new staged',binary:false});await second;pending[0]({path:'a.py',diff:'obsolete',binary:false});await first;
 expect(model.selected).toBe('b.py');expect(model.staged).toBe(true);expect(model.detail).toBe('new staged');
 const closed=model.select('a.py',false);model.close();pending[2]({path:'a.py',diff:'late after close',binary:false});await closed;expect(model.detail).toBe('');
 const wrong=model.select('a.py',false);pending[3]({path:'b.py',diff:'wrong source',binary:false});await wrong;expect(model.detail).toBe('');expect(model.notice).toContain('identity changed');
});
</ui:copy>

<ui:file path="apps/web/src/simple/GitReview.test.ts"><ui:paste select="#simple-git-review-browser-tests"/></ui:file>
## Review saved Git state while retaining an unsaved editor

Drive the actual View menu and engine over a temporary repository. A staged rename retains quoted and literal pathspec names; reading either diff side must not save an unsaved editor or change its identity and dimensions.

<ui:copy id="simple-git-review-browser-engine-witness">
"""Existing View menu, actual engine and temporary Git state; no source save."""
import importlib.util
import os
from pathlib import Path
import socket
import subprocess
import tempfile
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location("fixture",ROOT/"scripts/test-simple-browser.py");fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix="simple-git-review-") as temporary:
  folder=Path(temporary);root=folder/"workspace";root.mkdir()
  def git(*args):return subprocess.run(["git","-C",str(root),"--literal-pathspecs",*args],check=True,capture_output=True)
  source="value = 1\n";(root/"main.py").write_text(source);old='quoted "é -> .py';new=':(glob)*.py';(root/old).write_text("renamed = True\n")
  git("init","-q");git("add","--",".");git("-c","user.name=Fixture","-c","user.email=fixture@example.invalid","commit","-qm","base")
  (root/"removed.py").write_text("temporary\n");git("mv","--",old,new);saved="value = 2\n";(root/"main.py").write_text(saved)
  environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
   ui=f"http://127.0.0.1:{port}"
   engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
   vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path="main.py");driver.act("remember-editor",path="main.py");size=driver.act("editor-size",path="main.py")
   driver.act("source-tests-edit",path="main.py",text="# unsaved draft\n");draft=driver.act("search-source",path="main.py");assert "unsaved draft" in draft
   text=driver.act("review-open");assert new in text and old in text and "main.py" in text,text
   (root/"removed.py").unlink();assert driver.act("review-select",path="removed.py",staged=False)=="";assert "Git diff refused" in driver.act("review-status")
   diff=driver.act("review-select",path="main.py",staged=False);assert "+value = 2" in diff and "unsaved draft" not in diff,diff
   diff=driver.act("review-select",path=new,staged=True);assert "+renamed = True" in diff,diff
   diff=driver.act("review-select",path="main.py",staged=True);assert diff=="",diff
   driver.act("review-close");assert driver.act("editor-is-retained",path="main.py");assert driver.act("editor-size",path="main.py")==size;assert driver.act("search-source",path="main.py")==draft;assert (root/"main.py").read_text()==saved
   print("PASS: existing View menu, literal renamed paths, working-tree/index diffs, unsaved draft retained, stable code dimensions and unchanged disk")
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
</ui:copy>

<ui:file path="scripts/test-simple-git-review-browser.py"><ui:paste select="#simple-git-review-browser-engine-witness"/></ui:file>
</ui:doc>

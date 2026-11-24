<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Adapt a shared coverage surface without weakening source authority

For maintainers changing workspace services independently of the editor renderer.

The engine owns runners, retained results and measurement identities. This UI adapter owns transport, editor selection and navigation. Normalize JavaScript/TypeScript aliases into their existing collector family while leaving unsupported language IDs explicit. Keep positive reported positions separate from a file open: no observed execution-source change and a valid captured hash of the actual authored owner are required before requesting an exact reveal. A host must still check that hash against its current buffer.

<ui:copy id="coverage-surface-navigation">
import type {CoverageDiscovery,CoverageRun} from "./types";
export interface CoverageSourceNavigation {path:string;line:number|null;sourceHash:string|null;sourceState:"current"|"historical"|"unknown";runId:string|null}
export interface CoverageSurfaceClient {
  call:<T>(body:Record<string,unknown>)=>Promise<T>;
  select:(run:CoverageRun|null)=>void;
  navigate:(location:CoverageSourceNavigation,current:()=>boolean)=>boolean|void|Promise<boolean|void>;
  retireNavigation?:()=>void;
}
export function coverageLanguage(language:string){return ["javascript","javascriptreact","typescriptreact","jsx","tsx"].includes(language)?"typescript":language;}
/** A reported line needs a captured owner hash and no observed execution-source change. */
export function coverageSourceNavigation(run:CoverageRun|null,path:string,line:number|null,discovery:CoverageDiscovery|null=null):CoverageSourceNavigation {
  const mapping=line===null?undefined:run?.mappings[path]?.[line];
  const owner=mapping?.path??path,position=mapping?.line??line;
  const listing=discovery?.version===1?discovery:null;
  const hashes=run?.identity.source_hashes??listing?.source_hashes??{};
  const stable=run?run.source_changed_during_run===false:!!listing&&listing.source_changed_during_discovery===false;
  const observed=stable?hashes[owner]??null:null;
  const hash=observed&&/^[0-9a-f]{64}$/i.test(observed)?observed.toLowerCase():null;
  const precise=!!hash&&position!==null&&Number.isInteger(position)&&position>0;
  return {path:owner,line:precise?position:null,sourceHash:hash,sourceState:run?.source_state??listing?.source_state??"unknown",runId:run?.id??null};
}
</ui:copy>

Focus the witnesses on mapped ownership, changed execution source, unversioned discovery and unbound/invalid positions. A historical whole-project record can still identify an unchanged file; this does not make its entire scope current.

<ui:copy id="coverage-surface-navigation-witness">
import {expect,it} from "vitest";
import {coverageSourceNavigation,coverageLanguage} from "./CoverageSurface";
import type {CoverageDiscovery,CoverageRun} from "./types";
function run():CoverageRun {return {id:"record",mode:"automated",status:"completed",started_ms:0,project:".",language:"python",labels:[],group:null,source_state:"historical",identity:{collector:"provider",version:"1",configuration:"config",scope:".",source_hashes:{"app.py":"a".repeat(64),"note.md":"b".repeat(64)}},argv:[],output:"",error:null,exit_code:0,tests:[],evidence:null,metadata:{groups:{},manual:{},tests:{}},source_changed_during_run:false,manual_verdict:null,notes:"",checkpoints:[],parent:null,mappings:{"app.py":{1:{path:"note.md",line:4}}}};}
it("normalizes one collector family without pretending unsupported languages have a collector",()=>{expect(coverageLanguage("typescriptreact")).toBe("typescript");expect(coverageLanguage("go")).toBe("go");});
it("binds mapped owner locations to their own captured bytes, independently of global history",()=>{
  const record=run();expect(coverageSourceNavigation(record,"app.py",1)).toMatchObject({path:"note.md",line:4,sourceHash:"b".repeat(64),sourceState:"historical"});
  delete record.identity.source_hashes["note.md"];expect(coverageSourceNavigation(record,"app.py",1)).toMatchObject({path:"note.md",line:null,sourceHash:null});
  expect(coverageSourceNavigation(record,"app.py",null).line).toBeNull();
  record.identity.source_hashes["app.py"]="malformed";expect(coverageSourceNavigation(record,"app.py",2).line).toBeNull();
  record.source_changed_during_run=true;expect(coverageSourceNavigation(record,"app.py",2).line).toBeNull();
});
it("keeps changed or unversioned runner listings readable without positional authority",()=>{
  const discovery:CoverageDiscovery={version:1,tests:[],output:"",metadata:{groups:{},manual:{},tests:{}},project:".",language:"python",discovery:"runner",source_hashes:{"test.py":"c".repeat(64)},source_state:"current",source_changed_during_discovery:false,source_frozen:false,workspace_id:"workspace",fingerprint:"listing"};
  expect(coverageSourceNavigation(null,"test.py",2,discovery)).toMatchObject({line:2,sourceHash:"c".repeat(64)});
  discovery.source_changed_during_discovery=true;expect(coverageSourceNavigation(null,"test.py",2,discovery).line).toBeNull();
  expect(coverageSourceNavigation(null,"test.py",2,{...discovery,version:2} as unknown as CoverageDiscovery).sourceState).toBe("unknown");
  expect(coverageSourceNavigation(null,"test.py",-1,{...discovery,source_changed_during_discovery:false}).line).toBeNull();
});
</ui:copy>

<ui:file path="apps/web/src/coverage/CoverageSurface.test.ts"><ui:paste select="#coverage-surface-navigation-witness"/></ui:file>
<ui:file path="apps/web/src/coverage/CoverageSurface.ts"><ui:paste select="#coverage-surface-navigation"/></ui:file>
These view witnesses inject the product's own surface adapter to control reply ordering. They prove that a newer selection or a closed surface cannot reinstall old editor coverage, that pending history reads coalesce, and that fresh discovery replaces old result rows. A host navigation refusal keeps the report readable. They do not simulate or certify a third-party test runner.

The Kotlin witness opens the shared panel from an Android unit test. Its
configuration request uses the workspace directory, and runner actions
remain disabled. The native embedding uses the same control path.

<ui:copy id="coverage-surface-lifecycle-witness">
import {cleanup,render,screen,fireEvent,waitFor,act} from "@testing-library/react";
import {afterEach,expect,it,vi} from "vitest";
import {CoveragePanel} from "./CoverageTools";
import type {CoverageSurfaceClient} from "./CoverageSurface";
import type {CoverageDiscovery,CoverageRun} from "./types";
afterEach(cleanup);
function record(id:string):CoverageRun{return {id:id.repeat(32),mode:"automated",status:"completed",started_ms:1,project:".",language:"python",group:null,labels:[],identity:{collector:"provider",version:"1",configuration:"config",scope:".",source_hashes:{"app.py":"a".repeat(64)}},source_state:"current",source_changed_during_run:false,tests:[],output:"",error:null,exit_code:0,evidence:null,metadata:{groups:{},manual:{},tests:{}},manual_verdict:null,notes:"",argv:[],checkpoints:[],parent:null,mappings:{}};}
function deferred<T>(){let resolve!:(value:T)=>void;const promise=new Promise<T>(done=>resolve=done);return {promise,resolve};}
function client(read:(body:Record<string,unknown>)=>Promise<unknown>){return {call:vi.fn(read) as CoverageSurfaceClient["call"],select:vi.fn(),navigate:vi.fn()} satisfies CoverageSurfaceClient;}
it.each([false,true])("reads directory settings without a Kotlin runner (embedded=%s)",async embedded=>{
  const metadata={groups:{},manual:{},tests:{}};
  const surface=client(async body=>body.action==="list"?{runs:[]}:{metadata});
  render(<CoveragePanel close={()=>{}} client={surface} initialLanguage="kotlin"
    initialPath="app/src/test/kotlin/Test.kt" embedded={embedded}/>);
  expect((screen.getByLabelText("Project directory") as HTMLInputElement).value).toBe(".");
  expect(screen.getByText(/run the appropriate Gradle task/)).toBeTruthy();
  for(const name of ["Discover runner cases","Run saved project with coverage"]){
    expect((screen.getByRole("button",{name}) as HTMLButtonElement).disabled).toBe(true);
  }
  fireEvent.click(screen.getByRole("button",{name:"Read project groups and profiles"}));
  await waitFor(()=>expect(surface.call).toHaveBeenCalledWith({action:"inspect",path:".",language:"kotlin"}));
});
it("a newer selected run retires an older response and editor-selection effect",async()=>{
  const first=record("a"),second=record("b"),held=deferred<CoverageRun>();
  const surface=client(async body=>body.action==="list"?{runs:[first,second]}:body.id===first.id?held.promise:second);
  render(<CoveragePanel close={()=>{}} client={surface}/>);
  fireEvent.click(await screen.findByRole("button",{name:new RegExp(first.id)}));
  fireEvent.click(screen.getByRole("button",{name:new RegExp(second.id)}));
  await waitFor(()=>expect(surface.select).toHaveBeenCalledWith(second));
  await act(async()=>{held.resolve(first);await held.promise;});
  expect(surface.select).not.toHaveBeenCalledWith(first);expect(screen.getByText(second.id,{selector:"code"})).toBeTruthy();
});
it("closing retires late selection and deduplicates pending history reads",async()=>{
  const run=record("a"),history=deferred<{runs:CoverageRun[]}>(),held=deferred<CoverageRun>();
  const surface=client(body=>body.action==="list"?history.promise:held.promise);
  const view=render(<CoveragePanel close={()=>{}} client={surface}/>);
  fireEvent.click(screen.getByRole("button",{name:"Refresh runs"}));fireEvent.click(screen.getByRole("button",{name:"Refresh runs"}));
  expect(surface.call).toHaveBeenCalledTimes(1);
  await act(async()=>{history.resolve({runs:[run]});await history.promise;});
  fireEvent.click(await screen.findByRole("button",{name:new RegExp(run.id)}));view.unmount();
  await act(async()=>{held.resolve(run);await held.promise;});expect(surface.select).not.toHaveBeenCalled();
});
it("new discovery replaces old result rows and routes navigation through the host guard",async()=>{
  const run=record("a");run.tests=[{id:"old_case",status:"failed",output:"old failure",path:"app.py",line:1,labels:[]}];
  const discovery:CoverageDiscovery={version:1,tests:[{id:"new_case",status:"discovered",output:"",path:"app.py",line:2,labels:[]}],output:"new_case",metadata:run.metadata,project:".",language:"python",discovery:"runner",source_hashes:{"app.py":"a".repeat(64)},source_state:"current",source_changed_during_discovery:false,source_frozen:false,workspace_id:"root",fingerprint:"listing"};
  const surface=client(async body=>body.action==="list"?{runs:[run]}:body.action==="discover"?discovery:run);surface.navigate.mockRejectedValue(new Error("Source changed. Refresh the report."));
  render(<CoveragePanel close={()=>{}} client={surface} initialLanguage="python" embedded/>);
  fireEvent.click(await screen.findByRole("button",{name:new RegExp(run.id)}));await screen.findByText(/failed · old_case/);
  fireEvent.click(screen.getByRole("button",{name:"Discover runner cases"}));await screen.findByText(/discovered · new_case/);
  expect(screen.queryByText(/failed · old_case/)).toBeNull();expect(surface.select).toHaveBeenLastCalledWith(null);
  fireEvent.click(screen.getByText(/discovered · new_case/));fireEvent.click(screen.getByRole("button",{name:"Open source line 2"}));
  await screen.findByText("Error: Source changed. Refresh the report.");
  expect(surface.navigate).toHaveBeenCalledWith(expect.objectContaining({path:"app.py",line:2,sourceHash:"a".repeat(64),runId:null}),expect.any(Function));
  expect(screen.getByRole("dialog",{name:"Tests and code coverage"})).toBeTruthy();
});

it("keeps a report open after its host retires a source reveal",async()=>{
  const run=record("a"),held=deferred<boolean>(),close=vi.fn();
  run.tests=[{id:"case",status:"passed",output:"",path:"app.py",line:2,labels:[]}];
  const surface=client(async body=>body.action==="list"?{runs:[run]}:run);
  surface.navigate.mockReturnValueOnce(held.promise).mockResolvedValueOnce(true);
  render(<CoveragePanel close={close} client={surface} embedded/>);
  fireEvent.click(await screen.findByRole("button",{name:new RegExp(run.id)}));
  fireEvent.click(await screen.findByText(/passed · case/));
  fireEvent.click(screen.getByRole("button",{name:"Open source line 2"}));
  await act(async()=>{held.resolve(false);await held.promise;});
  expect(close).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button",{name:"Open source line 2"}));
  await waitFor(()=>expect(close).toHaveBeenCalledTimes(1));
});
it("refreshes the selected collector body when its summary finishes without replacing a newer selection",async()=>{
  const first=record("a"),second=record("b");first.status="running";let done=false;const held=deferred<CoverageRun>();
  const surface=client(async body=>body.action==="list"?{runs:[{...first,status:done?"completed":"running"},second]}:body.id===second.id?second:done?held.promise:first);
  render(<CoveragePanel close={()=>{}} client={surface}/>);fireEvent.click(await screen.findByRole("button",{name:new RegExp(first.id)}));await waitFor(()=>expect(surface.select).toHaveBeenCalledWith(first));
  done=true;fireEvent.click(screen.getByRole("button",{name:"Refresh runs"}));await waitFor(()=>expect(surface.call).toHaveBeenCalledWith({action:"get",id:first.id}));
  fireEvent.click(screen.getByRole("button",{name:new RegExp(second.id)}));await waitFor(()=>expect(surface.select).toHaveBeenCalledWith(second));const completed={...first,status:"completed",tests:[{id:"late_case",status:"passed",output:"",path:"app.py",line:1,labels:[]}]};await act(async()=>{held.resolve(completed);await held.promise;});expect(surface.select).not.toHaveBeenCalledWith(completed);expect(screen.queryByText(/late_case/)).toBeNull();
});
it("attaches real completed result rows after the selected run leaves running state",async()=>{
 const first=record("a");first.status="running";let done=false;const completed={...first,status:"failed",tests:[{id:"real_failure",status:"failed",output:"assert 1 == 2",path:"app.py",line:2,labels:[]}]};
 const surface=client(async body=>body.action==="list"?{runs:[done?completed:first]}:done?completed:first);render(<CoveragePanel close={()=>{}} client={surface}/>);fireEvent.click(await screen.findByRole("button",{name:new RegExp(first.id)}));await waitFor(()=>expect(surface.select).toHaveBeenCalledWith(first));done=true;fireEvent.click(screen.getByRole("button",{name:"Refresh runs"}));await screen.findByText(/failed · real_failure/);expect(surface.select).toHaveBeenLastCalledWith(completed);
});
it("native test names stay readable while retained identities are source qualified",async()=>{
 const run=record("c");run.tests=[{id:"js-v1:opaque-source-qualified-case",runner_name:"cohort negative",status:"failed",output:"native failure",path:"a.test.js",line:null,labels:[]},{id:"js-v1:empty",runner_name:"",status:"failed",output:"empty-name failure",path:"b.test.js",line:null,labels:[]}];const surface=client(async body=>body.action==="list"?{runs:[run]}:run);render(<CoveragePanel close={()=>{}} client={surface}/>);fireEvent.click(await screen.findByRole("button",{name:new RegExp(run.id)}));expect(await screen.findByText(/failed · cohort negative · a.test.js/)).toBeTruthy();expect(screen.getByText(/failed · "" · b.test.js/)).toBeTruthy();expect(screen.queryByText(/js-v1:opaque-source-qualified-case/)).toBeNull();
});
</ui:copy>

<ui:file path="apps/web/src/coverage/CoverageSurfaceView.test.tsx"><ui:paste select="#coverage-surface-lifecycle-witness"/></ui:file>
</ui:doc>

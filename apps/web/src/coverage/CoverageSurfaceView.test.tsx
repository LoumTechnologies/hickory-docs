import {cleanup,render,screen,fireEvent,waitFor,act} from "@testing-library/react";
import {afterEach,expect,it,vi} from "vitest";
import {CoveragePanel} from "./CoverageTools";
import type {CoverageSurfaceClient} from "./CoverageSurface";
import type {CoverageDiscovery,CoverageRun} from "./types";
afterEach(cleanup);
function record(id:string):CoverageRun{return {id:id.repeat(32),mode:"automated",status:"completed",started_ms:1,project:".",language:"python",group:null,labels:[],identity:{collector:"provider",version:"1",configuration:"config",scope:".",source_hashes:{"app.py":"a".repeat(64)}},source_state:"current",source_changed_during_run:false,tests:[],output:"",error:null,exit_code:0,evidence:null,metadata:{groups:{},manual:{},tests:{}},manual_verdict:null,notes:"",argv:[],checkpoints:[],parent:null,mappings:{}};}
function deferred<T>(){let resolve!:(value:T)=>void;const promise=new Promise<T>(done=>resolve=done);return {promise,resolve};}
function client(read:(body:Record<string,unknown>)=>Promise<unknown>){return {call:vi.fn(read) as CoverageSurfaceClient["call"],select:vi.fn(),navigate:vi.fn()} satisfies CoverageSurfaceClient;}
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

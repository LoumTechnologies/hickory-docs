<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/plainDebugHosts.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Debug controls preserve editor space](../../../../../../debug-controls-preserve-editor-space/contract.md)
- [Launch profiles choose targets within the viewed project](../../../../../contract.md)

Captured SHA-256: `6a3e4bc5221ac9d8f4b5a35e5fc363558e41c55e0a44140aa45cca011528a636`.

<book:entry path="apps/web/src/debug/plainDebugHosts.test.tsx">
import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { publishPausedElsewhere, resetPausedElsewhere } from "../lib/pausedElsewhere";
import { IDLE_SESSION } from "./useDebugger";
import { isWorkspaceSource, resetPlainDebugHosts, useFollowingDebugSession, usePublishDebugSession } from "./plainDebugHosts";

afterEach(()=>{cleanup();resetPlainDebugHosts();resetPausedElsewhere();});
it("keeps execution controls in a callee while stepping and running, including document-owned sessions",()=>{
  const step=vi.fn(),stop=vi.fn(),jumpTo=vi.fn(),runTo=vi.fn(),setSourceBreakpoints=vi.fn();
  const debug={...IDLE_SESSION,status:"paused" as const,step,stop,jumpTo,runTo,setSourceBreakpoints,pausedLine:null,
    frames:[{id:1,name:"helper",source:"helpers.py",source_line:2,in_document:false,line:null}]};
  const callee={...IDLE_SESSION,breakpoints:[{line:2,state:"bound" as const}]};
  const hook=renderHook(({session})=>{
    usePublishDebugSession("project.md",session);
    usePublishDebugSession("helpers.py",callee);
    return useFollowingDebugSession("helpers.py");
  },{initialProps:{session:debug as typeof IDLE_SESSION}});
  act(()=>publishPausedElsewhere("project.md",{path:"helpers.py",line:2}));
  expect(hook.result.current?.status).toBe("paused");
  expect(hook.result.current?.pausedLine).toBe(2);
  hook.result.current?.step("in");expect(step).toHaveBeenCalledWith("in");
  expect(setSourceBreakpoints).toHaveBeenCalledWith("hick:///helpers.py",callee.breakpoints);
  hook.result.current?.toggleBreakpoint(4);
  expect(setSourceBreakpoints).toHaveBeenLastCalledWith("hick:///helpers.py",[{line:2,state:"bound"},{line:4,state:"pending"}]);
  hook.result.current?.jumpTo(2);hook.result.current?.runTo(2);
  expect(jumpTo).not.toHaveBeenCalled();expect(runTo).not.toHaveBeenCalled();
  act(()=>publishPausedElsewhere("project.md",null));
  hook.rerender({session:{...debug,status:"running"}});
  expect(hook.result.current?.status).toBe("running");
  hook.result.current?.stop();expect(stop).toHaveBeenCalledOnce();
  hook.rerender({session:{...debug,status:"finished"}});
  expect(hook.result.current?.status).toBe("idle");
});
it("never opens a captured URI as a workspace-relative filename",()=>{
  expect(isWorkspaceSource("helpers.py")).toBe(true);
  expect(isWorkspaceSource("hick-project:///capture/helpers.py")).toBe(false);
  expect(isWorkspaceSource("/tmp/helpers.py")).toBe(false);
});

it("includes cleared file sets when collecting saved breakpoints for another launch",()=>{
  const start=vi.fn();
  const root={...IDLE_SESSION,start,sourceBreakpoints:{"hick:///helpers.py":[{line:2,state:"bound" as const}]}};
  const helper={...IDLE_SESSION,breakpoints:[]};
  const hook=renderHook(()=>{
    usePublishDebugSession("app.py",root);usePublishDebugSession("helpers.py",helper);
    return useFollowingDebugSession("app.py");
  });
  act(()=>hook.result.current?.start());
  expect(start).toHaveBeenCalledWith(undefined,{"hick:///helpers.py":[]},undefined,undefined);
});

// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
it("carries standing library breakpoints into a different compatible capture and preserves clearing",async()=>{
  const {acquireCapturedContext,capturedSourceKey}=await import("../lib/capturedContexts");
  const {rememberBreakpointPreferences}=await import("./breakpointPreferences");
  // Registration is exercised separately by capturedContexts.test; here use
  // the API mock to supply two native identities for different complete trees.
  const client=await import("../api/client");
  let count=0;
  const spy=vi.spyOn(client,"request").mockImplementation(async method=>method==="POST"?{id:`prefs${++count}`,uri:`hick-project:///prefs${count}/`,workspace:"/preferences-test"}:undefined);
  const entries=(value:number)=>({"app.py":{bytes:[value]},"library.py":{bytes:[100,101,102]}});
  const first=await acquireCapturedContext(entries(1),"project"),second=await acquireCapturedContext(entries(2),"project");
  const key=capturedSourceKey(`${first.uri}library.py`);
  const start=vi.fn();
  const session={...IDLE_SESSION,start};
  const hook=renderHook(()=>{usePublishDebugSession(`${second.uri}app.py`,session);return useFollowingDebugSession(`${second.uri}app.py`);});
  try {
    act(()=>rememberBreakpointPreferences(key,[{line:1,state:"pending",condition:"x > 0"}]));
    act(()=>hook.result.current?.start(undefined,{},undefined,{id:"app",fingerprint:"f"}));
    expect(start).toHaveBeenLastCalledWith(undefined,{[`${second.uri}library.py`]:[{line:1,state:"pending",condition:"x > 0",hit_condition:undefined,log_message:undefined}]},undefined,{id:"app",fingerprint:"f"});
    act(()=>rememberBreakpointPreferences(key,[]));
    act(()=>hook.result.current?.start());
    expect(start).toHaveBeenLastCalledWith(undefined,{[`${second.uri}library.py`]:[]},undefined,undefined);
  } finally {hook.unmount();first.release();second.release();spy.mockRestore();}
});
</book:entry>

</book:doc>

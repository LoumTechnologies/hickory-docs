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
  expect(start).toHaveBeenCalledWith(undefined,{"hick:///helpers.py":[]});
});

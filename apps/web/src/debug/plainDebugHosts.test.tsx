import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { publishPausedElsewhere, resetPausedElsewhere } from "../lib/pausedElsewhere";
import { IDLE_SESSION } from "./useDebugger";
import { isWorkspaceSource, resetPlainDebugHosts, useFollowingDebugSession, usePublishDebugSession } from "./plainDebugHosts";

afterEach(()=>{cleanup();resetPlainDebugHosts();resetPausedElsewhere();});
it("keeps execution controls in a callee while stepping and running, including document-owned sessions",()=>{
  const step=vi.fn(),stop=vi.fn(),jumpTo=vi.fn(),runTo=vi.fn();
  const debug={...IDLE_SESSION,status:"paused" as const,step,stop,jumpTo,runTo,pausedLine:null,
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
  expect(hook.result.current?.breakpoints[0]).toMatchObject({state:"pending",message:expect.stringContaining("another file")});
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

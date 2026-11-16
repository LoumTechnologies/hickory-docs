import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it,vi,beforeAll } from "vitest";
import {withDebugProtocol} from "./launchProfiles";
vi.mock("./launchProfiles",()=>({launchCatalogNotices:(catalog:{notices?:string[]})=>catalog.notices??[],withDebugProtocol:vi.fn((_query:unknown,effect:(catalog:{workspace:string})=>void)=>{effect({workspace:"fixture-root"});return Promise.resolve();})}));
import {savedDebugSources} from "./SourceGuards";
const fixtureSource=Array.from({length:100},(_,i)=>`line ${i}`).join('\n');
beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});vi.spyOn(savedDebugSources,'read').mockImplementation(uri=>({uri,workspace:'fixture-root',namespace:'fixture-root',identity:uri,text:fixtureSource,saved:true,current:()=>true}));});
async function settle(action:()=>void){await act(async()=>{action();await new Promise(resolve=>setTimeout(resolve,10));});}
import type { Realtime } from "../api/realtime";
import { DebugClient,DebugThreads,DebugLaunches,debugAdapterId } from "./client";
import { eventIsOurs, useDebugger, useDebuggerOver } from "./useDebugger";

/**
 * A socket that records what went out and can push frames back in.
 *
 * The point of testing through this rather than through `DebugClient`
 * directly: the bug this file exists for was that nothing ever registered the
 * client with the socket. Requests went out, every answer was dropped, and the
 * window sat on "starting…" — invisible to any test of the pieces on their own.
 */
class FakeSocket {
  readonly serverAuthoritative = true;
  sent: unknown[] = [];
  private handler: ((frame: Uint8Array) => boolean) | null = null;

  // The rest of the Realtime surface, which the debugger never touches.
  lsp() {
    return null;
  }
  onRunEvent() {
    return () => {};
  }
  bindDoc() {}
  whenSynced() {
    return Promise.resolve();
  }
  close() {}

  debug() {
    return {
      send: (frame: Uint8Array) => {
        this.sent.push(JSON.parse(new TextDecoder().decode(frame.subarray(1))));
      },
    };
  }

  onDebugFrame(handler: (frame: Uint8Array) => boolean) {
    this.handler = handler;
  }

  /** What the engine says, arriving as a real `0x03` frame. */
  deliver(event: Record<string, unknown>) {
    if(event.request_id===undefined&&(event.event==="started"||event.event==="build"||event.event==="failed"&&!event.session&&(event.about===undefined||event.about==="start"))){const start=[...this.sent].reverse().find(item=>(item as {op?:string}).op==="start") as {request_id?:string}|undefined;event={...event,request_id:start?.request_id};}
    if(event.event==='value'||event.event==='failed'&&event.session){const op=event.event==='value'?'eval':event.about;const last=[...this.sent].reverse().find(item=>(item as {op?:string;expression?:string}).op===op&&(op!=='eval'||(item as {expression?:string}).expression===event.expression||event.event==='failed')) as {request_id?:string;stop?:number;thread?:number;frame?:number}|undefined;event={...event,request_id:event.request_id??last?.request_id,...(event.event==='value'?{stop:event.stop??last?.stop,thread:event.thread??last?.thread,frame:event.frame??last?.frame}:{})};}
    if(event.event==='started'&&event.source_hash===undefined){const start=[...this.sent].reverse().find(item=>(item as {op?:string}).op==='start') as {source_hash?:string}|undefined;event={...event,source_hash:start?.source_hash};}
    if(event.event==='breakpoints'||event.event==='source_breakpoints'){const op=event.event==='breakpoints'?'breakpoints':'source_breakpoints',last=[...this.sent].reverse().find(item=>(item as {op?:string;source?:string}).op===op&&(op==='breakpoints'||(item as {source?:string}).source===event.source)) as {request_id?:string;source_hash?:string}|undefined;event={...event,request_id:event.request_id??last?.request_id,source_hash:event.source_hash??last?.source_hash};}
    const body = new TextEncoder().encode(JSON.stringify(event));
    const frame = new Uint8Array(body.length + 1);
    frame[0] = 0x03;
    frame.set(body, 1);
    this.handler?.(frame);
  }
}

const STARTED = {
  event: "started",
  session: "dbg-0",
  capabilities: {
    conditional_breakpoints: true,
    hit_conditional_breakpoints: true,
    log_points: true,
    set_variable: true,
    restart_frame: false,
    step_in_targets: true,
    goto_targets: true,
    step_back: false,
    exception_filters: [],
  },
  breakpoints: [{ line: 25, state: "bound" }],
};

const STOPPED = {
  event: "stopped",stop:1,thread:1,
  session: "dbg-0",
  reason: "breakpoint",
  line: 14,
  frames: [{ id: 2, name: "line_total", line: 14, source: "orders.py", in_document: true }],
  variables: [{ name: "quantity", value: "2", type_name: "int", variables_reference: 0 }],
};

function open() {
  const socket = new FakeSocket();
  const hook = renderHook(() => useDebugger(socket as unknown as Realtime, "orders.md"));
  return { socket, hook };
}

describe("the debugger, over the socket", () => {
  it("retains newly placed callee intentions without rebinding an existing launch, then includes them in a fresh source-bound launch",async()=>{
    const {socket,hook}=open();await settle(()=>hook.result.current.start());await settle(()=>{socket.deliver(STARTED);socket.deliver(STOPPED);});const caller=hook.result.current.breakpoints,count=socket.sent.length;
    await settle(()=>hook.result.current.setSourceBreakpoints("hick:///helpers.py",[{line:25,state:"pending",condition:"x > 1"}]));expect(socket.sent).toHaveLength(count);expect(hook.result.current.breakpoints).toEqual(caller);expect(hook.result.current.sourceBreakpoints["hick:///helpers.py"]).toEqual([]);
    await settle(()=>socket.deliver({event:"finished",session:"dbg-0"}));await settle(()=>hook.result.current.start());const start=socket.sent.at(-1) as {request_id:string;source_hashes:Record<string,string>};expect(start).toMatchObject({op:"start",sources:{"hick:///helpers.py":[{line:25,condition:"x > 1"}]}});expect(start.source_hashes['hick:///helpers.py']).toMatch(/^[a-f0-9]{64}$/);
    await settle(()=>{socket.deliver(STARTED);socket.deliver({event:'source_breakpoints',session:'dbg-0',source:'hick:///helpers.py',request_id:start.request_id,source_hash:start.source_hashes['hick:///helpers.py'],breakpoints:[{line:25,state:'bound',moved_to:26}]});});expect(hook.result.current.sourceBreakpoints['hick:///helpers.py']).toMatchObject([{line:26,requested_line:25,state:'bound',condition:'x > 1'}]);
    await settle(()=>hook.result.current.setSourceBreakpoints('hick:///helpers.py',[{line:25,state:'pending',condition:'x > 2'}]));expect(socket.sent.at(-1)).toMatchObject({op:'source_breakpoints',source:'hick:///helpers.py',source_hash:start.source_hashes['hick:///helpers.py'],breakpoints:[{line:25,condition:'x > 2'}]});
  });

  it("leaves 'starting' when the engine answers", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    expect(hook.result.current.status).toBe("starting");
    expect(socket.sent[0]).toMatchObject({ op: "start", doc: "hick:///orders.md" });

    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));
    expect(hook.result.current.capabilities?.goto_targets).toBe(true);
  });

  it("shows the paused line and the frame's values", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));

    await waitFor(() => expect(hook.result.current.status).toBe("paused"));
    expect(hook.result.current.pausedLine).toBe(14);
    expect(hook.result.current.frames[0].name).toBe("line_total");
    expect(hook.result.current.variables[0].value).toBe("2");
    // The top frame is selected, so evaluating goes to the right one.
    expect(hook.result.current.selectedFrame).toBe(2);
  });

  it("sends the session and exact current scope with every admitted request",async()=>{
    const {socket,hook}=open();await settle(()=>hook.result.current.start());await settle(()=>socket.deliver(STARTED));await settle(()=>socket.deliver(STOPPED));
    await settle(()=>hook.result.current.step('over'));expect(socket.sent.at(-1)).toMatchObject({op:'step',session:'dbg-0',how:'over',frame:2,stop:1,thread:1,request_id:expect.any(String)});
    await settle(()=>socket.deliver({...STOPPED,stop:2}));await settle(()=>hook.result.current.evaluate('quantity * 2'));expect(socket.sent.at(-1)).toMatchObject({op:'eval',session:'dbg-0',expression:'quantity * 2',context:'repl',frame:2,stop:2,thread:1,request_id:expect.any(String)});
    await settle(()=>hook.result.current.jumpTo(12));expect(socket.sent.at(-1)).toMatchObject({op:'jump',session:'dbg-0',line:12,stop:2,thread:1,source_hash:expect.any(String),request_id:expect.any(String)});
  });

  it("does nothing before a session exists", async() => {
    // Stepping with no session would be a request the engine answers with an
    // error, and the error would be the UI's fault.
    const { socket, hook } = open();
    await settle(() => hook.result.current.step("over"));
    await settle(() => hook.result.current.evaluate("x"));
    expect(socket.sent).toMatchObject([]);
  });

  it("keeps breakpoints across a session, and sends them when one starts", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.toggleBreakpoint(14));
    await settle(() => hook.result.current.start());
    expect(socket.sent.at(-1)).toMatchObject({ breakpoints: [{ line: 14 }] });

    await settle(() => socket.deliver({...STARTED,breakpoints:[{line:14,state:"bound",moved_to:25}]}));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));
    // The engine's answer replaces the optimistic set: it knows which bound.
    expect(hook.result.current.breakpoints).toEqual([{ line:25,requested_line:14,state:"bound" }]);
  });

  it("reports a failure instead of waiting forever", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() =>
      socket.deliver({
        event: "failed",
        message: "no debug adapter for python on this machine.",
      }),
    );
    await waitFor(() => expect(hook.result.current.status).toBe("failed"));
    expect(hook.result.current.message).toContain("no debug adapter");
  });

  it("carries a fetchable tool out of the failure, and drops it when one starts", async () => {
    // Protects docs/guarantees/debugging/a-missing-debugger-is-a-button.md
    //
    // The second half is the bug this pairing exists to prevent: installing
    // the adapter started the session successfully and left "No Python
    // debugger on this machine" sitting beside "finished — exit code 0".
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() =>
      socket.deliver({
        event: "failed",
        message: "no debug adapter for python on this machine.",
        offer_install: { kind: "dap", language: "python" },
      }),
    );
    await waitFor(() =>
      expect(hook.result.current.offerInstall).toEqual({ kind: "dap", language: "python" }),
    );

    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));
    expect(hook.result.current.offerInstall).toBeNull();
    expect(hook.result.current.message).toBeNull();
  });

  it("asks only about names the frame has", async () => {
    // Hovering prose, a tag name or a comment used to send the word to the
    // debugger and get a `NameError` back — an error about our question, not
    // about the program.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));
    const before = socket.sent.length;

    await act(async () => {
      expect(await hook.result.current.valueAt("order")).toBeNull();
      expect(await hook.result.current.valueAt("hick:file")).toBeNull();
    });
    expect(socket.sent.length).toBe(before);

    // A name that IS in the frame goes out, attribute access included.
    void hook.result.current.valueAt("quantity");
    void hook.result.current.valueAt("quantity.bit_length");
    expect(socket.sent.slice(before)).toMatchObject([
      { op: "eval", session: "dbg-0", expression: "quantity", context: "hover", frame: 2 },
      {
        op: "eval",
        session: "dbg-0",
        expression: "quantity.bit_length",
        context: "hover",
        frame: 2,
      },
    ]);
  });

  it("keeps a hover's failure out of the panel", async () => {
    // The panel is for what happened to the program. An unanswerable hover
    // resolves to nothing and the tooltip falls back to the type.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));

    let answered: string | null = "unset";
    await settle(() => {
      void hook.result.current.valueAt("quantity").then((v) => {
        answered = v;
      });
    });
    await settle(() => socket.deliver({ event: "failed",session:"dbg-0",about:"eval",message:"NameError: nope" }));
    await waitFor(() => expect(answered).toBeNull());
    expect(hook.result.current.message).toBeNull();
    // And the session is still alive: a hover failing is not a session failing.
    expect(hook.result.current.status).toBe("paused");
  });

  it("marks the breakpoint broken instead of shouting in the panel", async () => {
    // The reported bug: a refused `setBreakpoints` left the dot looking set,
    // put the reason in the panel, and left it there — so the next breakpoint
    // inherited a message about the previous one.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));
    await settle(() => hook.result.current.toggleBreakpoint(14));

    await settle(() =>
      socket.deliver({
        event: "failed",
        session: "dbg-0",
        about: "breakpoints",
        lines: [14],
        message: "the debug adapter refused `setBreakpoints`: Server disconnected unexpectedly",
      }),
    );

    await waitFor(() =>
      expect(hook.result.current.breakpoints.find((b) => b.line === 14)?.state).toBe("refused"),
    );
    // The reason lives on the breakpoint, for its hover.
    expect(hook.result.current.breakpoints.find((b) => b.line === 14)?.message).toContain(
      "disconnected",
    );
    // And nowhere else.
    expect(hook.result.current.message).toBeNull();
    // An unsolicited startup line is not an admitted intention or receipt.
    expect(hook.result.current.breakpoints.find((b) => b.line === 25)).toBeUndefined();
  });

  it("keeps a breakpoint local once the program has ended", async () => {
    // Setting one after the program finished used to send it to a session
    // that could only refuse: the breakpoint is for the next run.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver({ event: "finished", session: "dbg-0" }));
    await waitFor(() => expect(hook.result.current.status).toBe("finished"));
    const before = socket.sent.length;

    await settle(() => hook.result.current.toggleBreakpoint(14));
    expect(socket.sent.length).toBe(before);
    expect(hook.result.current.breakpoints.some((b) => b.line === 14)).toBe(true);
  });

  it("re-asks every watch at each pause and frame change", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));
    await waitFor(() => expect(hook.result.current.status).toBe("paused"));

    // Adding a watch while paused asks straight away, in `watch` context so
    // the adapter treats it as something evaluated repeatedly.
    await settle(() => hook.result.current.addWatch("quantity * 2"));
    expect(socket.sent.at(-1)).toMatchObject({
      op: "eval",
      session: "dbg-0",
      expression: "quantity * 2",
      context: "watch",
      frame: 2,
    });

    // The answer correlates by the expression it echoes back.
    await settle(() =>
      socket.deliver({
        event: "value",
        session: "dbg-0",
        expression: "quantity * 2",
        value: "4",
        type: "int",
        reference: 0,
      }),
    );
    await waitFor(() =>
      expect(hook.result.current.watches).toEqual([{ expression: "quantity * 2", value: "4" }]),
    );

    // A step that pauses again re-asks, unprompted: a watch showing the value
    // from two steps ago is worse than no watch at all.
    await settle(() => hook.result.current.step("over"));
    const before = socket.sent.length;
    await settle(() => socket.deliver({ ...STOPPED,stop:2, line:15 }));
    await waitFor(() =>
      expect(
        socket.sent
          .slice(before)
          .some((sent) => (sent as { context?: string }).context === "watch"),
      ).toBe(true),
    );
  });

  it("keeps watch expressions but drops their values when the program ends", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.addWatch("quantity"));
    await settle(() => hook.result.current.addWatch("quantity")); // and never twice
    expect(hook.result.current.watches).toEqual([{ expression: "quantity", value: null }]);

    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));
    await settle(() =>
      socket.deliver({
        event: "value",
        session: "dbg-0",
        expression: "quantity",
        value: "2",
        type: "int",
        reference: 0,
      }),
    );
    await waitFor(() => expect(hook.result.current.watches[0].value).toBe("2"));

    // The expression is for the next run; the value belonged to this one.
    await settle(() => socket.deliver({ event: "finished", session: "dbg-0" }));
    await waitFor(() => expect(hook.result.current.watches).toEqual([
      { expression: "quantity", value: null },
    ]));

    await settle(() => hook.result.current.removeWatch("quantity"));
    expect(hook.result.current.watches).toEqual([]);
  });

  it("answers a typed expression as a promise, without the known-name guard", async () => {
    // `valueAt` refuses names the frame does not have (it exists for hover);
    // the inline eval is a person typing, and any expression is fair.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));
    await waitFor(() => expect(hook.result.current.status).toBe("paused"));

    let answered: string | null = null;
    await settle(() => {
      void hook.result.current.query("len('abc')").then((value) => {
        answered = value;
      });
    });
    expect(socket.sent.at(-1)).toMatchObject({
      op: "eval",
      session: "dbg-0",
      expression: "len('abc')",
      context: "repl",
      frame: 2,
    });
    await settle(() =>
      socket.deliver({
        event: "value",
        session: "dbg-0",
        expression: "len('abc')",
        value: "3",
        type: "int",
        reference: 0,
      }),
    );
    await waitFor(() => expect(answered).toBe("3"));
  });

  it("forgets the session when the program finishes, and sends it nothing more", async () => {
    // The bug's visible half: the program ran to its end, the server reaped
    // the session — and the UI kept the id, a live-looking strip, and a Stop
    // that earned an error from a session that no longer existed.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));
    await settle(() => socket.deliver({ event: "finished", session: "dbg-0", exit_code: 0 }));

    await waitFor(() => expect(hook.result.current.status).toBe("finished"));
    // The end says how it ended, and the paused chrome is gone.
    expect(hook.result.current.exitCode).toBe(0);
    expect(hook.result.current.pausedLine).toBeNull();
    expect(hook.result.current.frames).toEqual([]);
    expect(hook.result.current.variables).toEqual([]);

    // No request may reach the dead session: not a step, not an eval, not a
    // hover — each would be answered only by an error about a gone session.
    const before = socket.sent.length;
    await settle(() => hook.result.current.step("over"));
    await settle(() => hook.result.current.evaluate("quantity"));
    await settle(() => hook.result.current.jumpTo(12));
    await act(async () => {
      expect(await hook.result.current.valueAt("quantity")).toBeNull();
      expect(await hook.result.current.query("1 + 1")).toBeNull();
    });
    expect(socket.sent.length).toBe(before);

    // Stop is now a dismissal: nothing goes out, the strip clears locally.
    await settle(() => hook.result.current.stop());
    expect(socket.sent.length).toBe(before);
    await waitFor(() => expect(hook.result.current.status).toBe("idle"));
  });

  it("does not send a stop for an already-finished session on unmount", async () => {
    // The unmount cleanup exists for a window closed MID-session. After
    // "finished" there is nothing to stop, and sending one anyway is a
    // request to a session the server already reaped.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver({ event: "finished", session: "dbg-0", exit_code: 0 }));
    await waitFor(() => expect(hook.result.current.status).toBe("finished"));

    const before = socket.sent.length;
    hook.unmount();
    expect(socket.sent.length).toBe(before);
  });

  it("keeps 'finished' when a stray failure arrives afterwards", async () => {
    // A hover or watch answered late, after the program ended, must not
    // redress a clean end as a broken one.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver({ event: "finished", session: "dbg-0", exit_code: 0 }));
    await waitFor(() => expect(hook.result.current.status).toBe("finished"));

    await settle(() => socket.deliver({ event: "failed", session: "dbg-0", message: "too late" }));
    expect(hook.result.current.status).toBe("finished");
  });

  it("goes back to idle when the session ends", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await settle(() => socket.deliver(STOPPED));
    await settle(() => socket.deliver({ event: "ended", session: "dbg-0" }));

    await waitFor(() => expect(hook.result.current.status).toBe("idle"));
    expect(hook.result.current.pausedLine).toBeNull();
    expect(hook.result.current.frames).toEqual([]);
  });
});

// Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
describe("two panes on one socket", () => {
  const frame = (event: Record<string, unknown>) => {
    const body = new TextEncoder().encode(JSON.stringify(event));
    const out = new Uint8Array(body.length + 1);
    out[0] = 0x03;
    out.set(body, 1);
    return out;
  };

  it("each takes only its own session", async () => {
    // Every plain file's pane shares the workspace connection, so the
    // stream carries everybody's sessions. A pane that took a neighbour's
    // `stopped` would show itself paused in a program it never started.
    const sent: unknown[] = [];
    const client = new DebugClient({
      send: (bytes) => sent.push(JSON.parse(new TextDecoder().decode(bytes.subarray(1)))),
    });
    const a = renderHook(() => useDebuggerOver(client, "tools/a.py"));
    const b = renderHook(() => useDebuggerOver(client, "tools/b.py"));

    await settle(() => a.result.current.start());
    expect(sent[0]).toMatchObject({ op: "start", doc: "hick:///tools/a.py" });
    await settle(() => {
      client.handleFrame(frame({ ...STARTED, doc: "hick:///tools/a.py",request_id:(sent[0] as {request_id:string}).request_id }));
    });
    await waitFor(() => expect(a.result.current.status).toBe("running"));
    expect(b.result.current.status).toBe("idle");

    await settle(() => {
      client.handleFrame(frame(STOPPED));
    });
    await waitFor(() => expect(a.result.current.status).toBe("paused"));
    expect(b.result.current.status).toBe("idle");
    expect(b.result.current.pausedLine).toBeNull();

    // A start that fails names the file it was for, and lands there only.
    await settle(() => b.result.current.start());
    await settle(() => {
      client.handleFrame(
        frame({
          event: "failed",
          session: null,
          doc: "hick:///tools/b.py",
          message: "no debug adapter for python on this machine",
          about: "start",request_id:(sent.at(-1) as {request_id:string}).request_id,
        }),
      );
    });
    await waitFor(() => expect(b.result.current.status).toBe("failed"));
    expect(a.result.current.status).toBe("paused");
    expect(a.result.current.message).toBeNull();
  });

  it("still takes an answer that names no file", async() => {
    // A document's own socket carries one session and an older engine may
    // not say which; that must keep working.
    expect(eventIsOurs({ ...STARTED, event: "started" } as never, "hick:///x.md", null)).toBe(true);
    expect(eventIsOurs(STOPPED as never, "hick:///x.md", "dbg-0")).toBe(true);
    expect(eventIsOurs(STOPPED as never, "hick:///x.md", null)).toBe(false);
    expect(eventIsOurs(STOPPED as never, "hick:///x.md", "dbg-9")).toBe(false);
  });
});


// Protects docs/guarantees/debugging/a-breakpoint-can-carry-a-condition.md
//
// `hick-dap::Breakpoint` has carried `condition`, `hit_condition` and
// `log_message` since the debugger was written, and the server deserializes
// all three. Nothing could set one: the hook sent `{ line }` and nothing else,
// and `debugStateEffects` hardcoded `conditional: false`.
describe("a breakpoint carries what the person attached to it", () => {
  it("sends a condition on the wire, and only what was set", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    await settle(() => hook.result.current.toggleBreakpoint(10));
    await settle(() => hook.result.current.setBreakpointCondition(10, { condition: "i > 10" }));

    const last = socket.sent.at(-1) as { op: string; breakpoints: Record<string, unknown>[] };
    expect(last.op).toBe("breakpoints");
    const sent = last.breakpoints.find((b) => b.line === 10)!;
    expect(sent).toEqual({ line: 10, condition: "i > 10" });
    // An adapter must never be handed an empty string to parse.
    expect(sent).not.toHaveProperty("hit_condition");
    expect(sent).not.toHaveProperty("log_message");
  });

  it("carries a hit count and a log message too", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    await settle(() => hook.result.current.toggleBreakpoint(4));
    await settle(() =>
      hook.result.current.setBreakpointCondition(4, {
        hit_condition: "> 5",
        log_message: "here with i={i}",
      }),
    );
    const last = socket.sent.at(-1) as { breakpoints: Record<string, unknown>[] };
    expect(last.breakpoints.find((b) => b.line === 4)).toEqual({
      line: 4,
      hit_condition: "> 5",
      log_message: "here with i={i}",
    });
  });

  it("clears a condition when it is emptied", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    await settle(() => hook.result.current.toggleBreakpoint(7));
    await settle(() => hook.result.current.setBreakpointCondition(7, { condition: "x" }));
    await settle(() => hook.result.current.setBreakpointCondition(7, { condition: "  " }));

    const last = socket.sent.at(-1) as { breakpoints: Record<string, unknown>[] };
    expect(last.breakpoints.find((b) => b.line === 7)).toEqual({ line: 7 });
    expect(hook.result.current.breakpoints.find((b) => b.line === 7)?.condition).toBeUndefined();
  });

  it("keeps the condition when the adapter moves the breakpoint", async () => {
    // The subtle one. A breakpoint that slides to the next executable line
    // and silently loses its condition would stop on every pass — the exact
    // opposite of what was asked for.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    await settle(() => hook.result.current.toggleBreakpoint(12));
    await settle(() => hook.result.current.setBreakpointCondition(12, { condition: "n == 3" }));
    await settle(() =>
      socket.deliver({
        event: "breakpoints",
        session: "dbg-0",
        breakpoints: [{ line: 12, state: "bound", moved_to: 13 }],
      }),
    );

    const held = hook.result.current.breakpoints;
    expect(held).toHaveLength(1);
    expect(held[0].line).toBe(13);
    expect(held[0].condition).toBe("n == 3");
  });

  it("keeps a condition set before the program runs, and starts with it", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.toggleBreakpoint(3));
    await settle(() => hook.result.current.setBreakpointCondition(3, { condition: "ready" }));
    // Nothing is running, so nothing was sent — the condition is for the run.
    expect(socket.sent).toHaveLength(0);

    await settle(() => hook.result.current.start());
    expect(socket.sent[0]).toMatchObject({
      op: "start",
      breakpoints: [{ line: 3, condition: "ready" }],
    });
  });
});

// Protects docs/guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters.md
describe("exception breakpoints", () => {
  it("sends the filters that are on, by id", async () => {
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));

    await settle(() => hook.result.current.toggleExceptionFilter("uncaught"));
    expect(socket.sent.at(-1)).toMatchObject({
      op: "exception_breakpoints",
      session: "dbg-0",
      filters: ["uncaught"],
    });

    await settle(() => hook.result.current.toggleExceptionFilter("raised"));
    expect(socket.sent.at(-1)).toMatchObject({ filters: ["uncaught", "raised"] });

    await settle(() => hook.result.current.toggleExceptionFilter("uncaught"));
    expect(socket.sent.at(-1)).toMatchObject({ filters: ["raised"] });
    expect(hook.result.current.exceptionFilters).toEqual(["raised"]);
  });

  it("keeps them across runs and re-applies them to the next session", async () => {
    // A standing "stop on uncaught" that quietly stopped applying on the
    // second run would be worse than never having offered it.
    const { socket, hook } = open();
    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));
    await settle(() => hook.result.current.toggleExceptionFilter("uncaught"));

    await settle(() => socket.deliver({ event: "finished", session: "dbg-0", exit_code: 0 }));
    await waitFor(() => expect(hook.result.current.status).toBe("finished"));
    expect(hook.result.current.exceptionFilters).toEqual(["uncaught"]);

    await settle(() => hook.result.current.start());
    await settle(() => socket.deliver({ ...STARTED, session: "dbg-1" }));
    await waitFor(() =>
      expect(socket.sent).toContainEqual({
        op: "exception_breakpoints",
        session: "dbg-1",
        filters: ["uncaught"],
      }),
    );
  });
});

// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
it("keeps the clicked source context for restart and replaces it for a different surface", async() => {
  const { socket, hook } = open();
  const context = 'latest-file:["base","<hick:project/>source"]';
  await settle(() => hook.result.current.start("app.py", {}, context));
  expect(socket.sent.at(-1)).toMatchObject({ op: "start", program: "app.py", project: context });
  await settle(() => hook.result.current.start("app.py"));
  expect(socket.sent.at(-1)).toMatchObject({ project: context });
  await settle(() => hook.result.current.start("other.py", {}, "new"));
  expect(socket.sent.at(-1)).not.toHaveProperty("project");
  hook.unmount();
});


describe("source-current debugger thread observations",()=>{
 it("admits only the latest exact stopped list and never selects optimistically",async()=>{
  const model=new DebugThreads();model.stopped("session",3,1);const first=model.read()!,second=model.read()!;
  const event=(request:string,stop=3,selected=1)=>({event:"threads" as const,session:"session",stop,selected,request_id:request,threads:[{id:1,name:"main"},{id:2,name:"worker"}]});
  expect(model.accept(event(first.request))).toBe(false);expect(model.accept(event(second.request,2))).toBe(false);expect(model.accept({...event(second.request),session:"foreign"})).toBe(false);expect(model.accept(event(second.request))).toBe(true);
  const selected=model.select(2)!;expect(selected.stop).toBe(3);expect(model.scope?.thread).toBe(1);expect(model.select(1)).toBeNull();model.stopped("session",4,2);expect(model.reading).toBeNull();expect(model.accept(event(selected.request,4,2))).toBe(true);expect(model.scope?.thread).toBe(2);
  model.clear();expect(model.accept(event(selected.request,4,2))).toBe(false);expect(model.select(2)).toBeNull();
 });
 it("rejects malformed identities and retires unrelated stop generations while keeping optional failures local",async()=>{
  const model=new DebugThreads();model.stopped("session",1,1);const read=model.read()!,event={event:"threads" as const,session:"session",stop:1,selected:1,request_id:read.request,threads:[{id:1,name:"main"}]};expect(model.accept(event)).toBe(true);
  const refresh=model.read()!;expect(model.failed("provider unavailable","old request")).toBe(false);expect(model.busy).toBe(true);expect(model.failed("provider unavailable",refresh.request)).toBe(true);expect(model.reading?.threads).toEqual([{id:1,name:"main"}]);
  const next=model.read()!;expect(model.accept({...event,request_id:next.request,threads:[{id:1,name:"a"},{id:1,name:"b"}]})).toBe(false);expect(model.reading?.threads).toEqual([{id:1,name:"main"}]);model.stopped("session",2,1);expect(model.accept({...event,request_id:next.request})).toBe(false);
  expect(model.stopped("session",Number.MAX_SAFE_INTEGER+1,1)).toBe(false);expect(model.scope).toEqual({session:"session",stop:2,thread:1});
 });
 it("routes wire thread reads and selection through the hook without failing the paused stack",async()=>{
  const {socket,hook}=open(),result=hook.result;await settle(()=>result.current.start());await settle(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:7,thread:1});});await waitFor(()=>expect(result.current.status).toBe("paused"));
  const frames=result.current.frames;await settle(()=>result.current.readThreads?.());const request=socket.sent.at(-1) as {op:string;request_id:string};expect(request.op).toBe("threads");
  await settle(()=>socket.deliver({event:"failed",session:"dbg-0",about:"threads",request_id:request.request_id,message:"adapter read unavailable"}));expect(result.current.status).toBe("paused");expect(result.current.frames).toBe(frames);expect(result.current.threadMessage).toContain("unavailable");
  await settle(()=>result.current.readThreads?.());const latest=socket.sent.at(-1) as {request_id:string};await settle(()=>socket.deliver({event:"threads",session:"dbg-0",stop:7,selected:1,request_id:latest.request_id,threads:[{id:1,name:"main"},{id:2,name:"worker"}]}));await waitFor(()=>expect(result.current.threads).toHaveLength(2));
  await settle(()=>result.current.selectThread?.(2));const selection=socket.sent.at(-1) as {op:string;stop:number;request_id:string};expect(selection.op).toBe("select_thread");expect(selection.stop).toBe(7);expect(result.current.selectedThread).toBe(1);const sent=socket.sent.length;await settle(()=>{result.current.step("over");result.current.readThreads?.();result.current.selectFrame(2);});expect(socket.sent).toHaveLength(sent);expect(await result.current.query("quantity")).toBeNull();
  await settle(()=>socket.deliver({...STOPPED,stop:8,thread:2}));await settle(()=>socket.deliver({event:"threads",session:"dbg-0",stop:8,selected:2,request_id:selection.request_id,threads:[{id:1,name:"main"},{id:2,name:"worker"}]}));await waitFor(()=>expect(result.current.selectedThread).toBe(2));expect(result.current.status).toBe("paused");
 });
 it("correlates values to the requested stop, thread, frame and request before settling current expressions",async()=>{
  const {socket,hook}=open(),result=hook.result;await settle(()=>result.current.start());await settle(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:10,thread:1});});
  let value!:Promise<string|null>;await settle(()=>{value=result.current.query("quantity");});const first=socket.sent.at(-1) as {request_id:string;stop:number;thread:number};expect(first.stop).toBe(10);expect(first.thread).toBe(1);
  const reply={event:"value",session:"dbg-0",expression:"quantity",value:"2",type_name:"int",type:"legacy",reference:0,frame:2,thread:1,stop:10,request_id:first.request_id};
  await settle(()=>socket.deliver({...reply,stop:9}));expect(result.current.lastValue).toBeNull();await settle(()=>socket.deliver({...reply,frame:99}));expect(result.current.lastValue).toBeNull();await act(async()=>{socket.deliver(reply);expect(await value).toBe("2");});expect(result.current.lastValue?.type).toBe("int");
  let stale!:Promise<string|null>;await settle(()=>{stale=result.current.query("quantity");});const old=socket.sent.at(-1) as {request_id:string};await act(async()=>{socket.deliver({...STOPPED,stop:11,thread:2});expect(await stale).toBeNull();});
  await settle(()=>socket.deliver({...reply,request_id:old.request_id}));expect(result.current.lastValue).toBeNull();
  await settle(()=>{value=result.current.query("quantity");});const latest=socket.sent.at(-1) as {request_id:string};await settle(()=>socket.deliver({event:"failed",session:"dbg-0",about:"eval",request_id:old.request_id,message:"old evaluation failed"}));await act(async()=>{socket.deliver({...reply,stop:11,thread:2,request_id:latest.request_id,value:"worker"});expect(await value).toBe("worker");});
 });

 it("does not relabel top-frame locals as another frame and keeps evaluation refusal local",async()=>{
  const {socket,hook}=open(),result=hook.result;await settle(()=>result.current.start());const frames=[...STOPPED.frames,{id:3,name:"worker",line:8,source:"orders.py",in_document:true}];await settle(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:30,thread:2,frames});});
  await settle(()=>result.current.selectFrame(3));expect(result.current.selectedFrame).toBe(3);expect(result.current.variables).toEqual([]);let value!:Promise<string|null>;await settle(()=>{value=result.current.query("worker_value");});const query=socket.sent.at(-1) as {request_id:string;frame:number};expect(query.frame).toBe(3);
  await act(async()=>{socket.deliver({event:"failed",session:"dbg-0",about:"eval",request_id:query.request_id,message:"Unable to find thread for evaluation"});expect(await value).toBeNull();});expect(result.current.status).toBe("paused");expect(result.current.frames).toEqual(frames);expect(result.current.selectedThread).toBe(2);
  await settle(()=>result.current.selectFrame(2));expect(result.current.variables).toEqual(STOPPED.variables);expect(result.current.status).toBe("paused");
 });

 it("admits only current selected-frame locals and keeps failed optional scopes separate from the stack",async()=>{
  const {socket,hook}=open(),result=hook.result,frames=[...STOPPED.frames,{id:3,name:"worker",line:8,source:"orders.py",in_document:true}];await settle(()=>result.current.start());await settle(()=>{socket.deliver({...STARTED,capabilities:{...STARTED.capabilities,frame_locals:true}});socket.deliver({...STOPPED,stop:40,thread:2,frames});});
  await settle(()=>result.current.selectFrame(3));const request=socket.sent.at(-1) as {op:string;request_id:string;stop:number;thread:number;frame:number};expect(request).toMatchObject({op:"state",stop:40,thread:2,frame:3});expect(result.current.frameVariablesPending).toBe(true);expect(result.current.variables).toEqual([]);
  const response={event:"frame_variables",session:"dbg-0",stop:40,thread:2,frame:3,request_id:request.request_id,variables:[{name:"worker_value",value:"worker",type:"str",variables_reference:0}]};await settle(()=>socket.deliver({...response,frame:2}));expect(result.current.variables).toEqual([]);await settle(()=>socket.deliver(response));expect(result.current.variables).toEqual(response.variables);expect(result.current.frameVariablesPending).toBe(false);
  await settle(()=>result.current.selectFrame(2));const top=socket.sent.at(-1) as {request_id:string};await settle(()=>socket.deliver({event:"failed",session:"dbg-0",about:"state",request_id:top.request_id,message:"adapter scopes unavailable"}));expect(result.current.status).toBe("paused");expect(result.current.frames).toEqual(frames);expect(result.current.variables).toEqual(STOPPED.variables);expect(result.current.frameVariablesMessage).toContain("unavailable");
  await settle(()=>result.current.step("over"));await settle(()=>socket.deliver(response));expect(result.current.status).toBe("running");expect(result.current.frameVariablesPending).toBe(false);
 });

 it("keeps captured frame lifetimes independent of locals arrival and rejects retired stack navigation",async()=>{
  const {socket,hook}=open(),result=hook.result;await settle(()=>result.current.start());await settle(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:50,thread:1});});const captured=result.current.captureFrame?.(2)!;expect(captured.current()).toBe(true);
  await settle(()=>socket.deliver({event:"run_output",session:"dbg-0",events:[{category:"stdout",text:"independent output"}]}));expect(captured.current()).toBe(true);
  await settle(()=>socket.deliver({...STOPPED,stop:51,thread:1,frames:STOPPED.frames,variables:STOPPED.variables,stack_error:"stack provider unavailable"}));expect(captured.current()).toBe(false);expect(result.current.frames).toEqual([]);expect(result.current.historicalFrames).toEqual(STOPPED.frames);expect(result.current.stackMessage).toContain("unavailable");const sent=socket.sent.length;await settle(()=>result.current.selectFrame(2));expect(socket.sent).toHaveLength(sent);expect(result.current.captureFrame?.(2)).toBeNull();expect(result.current.status).toBe("paused");
 });

 it("does not regress the entire pause from held older stops and preserves selected caller on same-stop refresh",async()=>{
  const {socket,hook}=open(),result=hook.result,frames=[...STOPPED.frames,{id:3,name:"caller",line:7,source:"orders.py",in_document:true}];await settle(()=>result.current.start());await settle(()=>{socket.deliver(STARTED);socket.deliver({...STOPPED,stop:60,thread:1,frames});});await settle(()=>result.current.selectFrame(3));const captured=result.current.captureFrame?.(3)!;
  await settle(()=>socket.deliver({...STOPPED,stop:60,thread:1,frames:frames.map(frame=>({...frame}))}));expect(result.current.selectedFrame).toBe(3);expect(result.current.pausedLine).toBe(7);expect(result.current.variables).toEqual([]);expect(captured.current()).toBe(true);
  const accepted=result.current.frames;await settle(()=>socket.deliver({...STOPPED,stop:59,thread:2,frames:[],variables:[],line:null}));expect(result.current.frames).toBe(accepted);expect(result.current.selectedFrame).toBe(3);expect(result.current.selectedThread).toBe(1);
  await settle(()=>result.current.step("over"));await settle(()=>socket.deliver({...STOPPED,stop:60,thread:1,frames}));expect(result.current.status).toBe("running");expect(captured.current()).toBe(false);
 });
 it("binds launch receipts before sending and cleans only known retired same-file launches",async()=>{
  const {socket,hook}=open(),result=hook.result;await settle(()=>result.current.start());const first=socket.sent.at(-1) as {request_id:string};expect(first.request_id).toBeTruthy();await settle(()=>result.current.stop());await settle(()=>result.current.start());const second=socket.sent.at(-1) as {request_id:string};expect(second.request_id).not.toBe(first.request_id);
  const before=socket.sent.length;await settle(()=>socket.deliver({...STARTED,session:"foreign",request_id:"another caller"}));expect(socket.sent).toHaveLength(before);expect(result.current.status).toBe("starting");
  await settle(()=>socket.deliver({...STARTED,session:"old",request_id:first.request_id}));expect(socket.sent.at(-1)).toMatchObject({op:"stop",session:"old"});expect(result.current.status).toBe("starting");
  await settle(()=>socket.deliver({event:"build",doc:"hick:///orders.md",request_id:first.request_id,events:[{type:"stdout",text:"old build"}]}));expect(result.current.buildOutput).toEqual([]);
  await settle(()=>socket.deliver({...STARTED,session:"new",request_id:second.request_id}));await settle(()=>socket.deliver({...STOPPED,session:"new",stop:1,thread:1}));const frames=result.current.frames;await settle(()=>socket.deliver({...STARTED,session:"new",request_id:second.request_id}));expect(result.current.status).toBe("paused");expect(result.current.frames).toBe(frames);
  await settle(()=>socket.deliver({event:"failed",session:null,about:"start",request_id:first.request_id,message:"old failed start"}));expect(result.current.status).toBe("paused");expect(result.current.message).toBeNull();
  const launches=new DebugLaunches(),old=launches.begin();for(let index=0;index<70;index++)launches.begin();expect(launches.started(old,"old")).toBe("foreign");
 });

 it("retains DAP zero and negative signed-int32 thread/frame identities without sentinel guessing",async()=>{
  const model=new DebugThreads();expect(model.stopped("scope",1,0)).toBe(true);const read=model.read()!;expect(model.accept({event:"threads",session:"scope",stop:1,selected:0,request_id:read.request,threads:[{id:0,name:"main"},{id:-7,name:"worker"}]})).toBe(true);expect(model.select(-7)?.thread).toBe(-7);expect(model.stopped("scope",2,-7)).toBe(true);expect(model.scope?.thread).toBe(-7);expect(debugAdapterId(true)).toBe(false);expect(debugAdapterId(0.5)).toBe(false);expect(debugAdapterId(2147483648)).toBe(false);
  const {socket,hook}=open(),result=hook.result,frames=[{...STOPPED.frames[0],id:0},{...STOPPED.frames[0],id:-3,name:"caller"}];await settle(()=>result.current.start());await settle(()=>{socket.deliver({...STARTED,capabilities:{...STARTED.capabilities,frame_locals:true}});socket.deliver({...STOPPED,stop:70,thread:0,frames});});expect(result.current.selectedFrame).toBe(0);expect(result.current.selectedThread).toBe(0);
  let value!:Promise<string|null>;await settle(()=>{value=result.current.query("quantity");});const expression=socket.sent.at(-1) as {request_id:string};expect(socket.sent.at(-1)).toMatchObject({thread:0,frame:0,stop:70});await act(async()=>{socket.deliver({event:"value",session:"dbg-0",expression:"quantity",value:"zero frame",type:null,reference:0,stop:70,thread:0,frame:0,request_id:expression.request_id});expect(await value).toBe("zero frame");});
  await settle(()=>result.current.selectFrame(-3));expect(socket.sent.at(-1)).toMatchObject({op:"state",frame:-3,thread:0});expect(result.current.captureFrame?.(-3)?.current()).toBe(true);
 });

 it("binds opaque observation scopes after Reading-first delivery and never downgrades to held old evidence",async()=>{
  const {socket,hook}=open(),result=hook.result;await settle(()=>result.current.start());await settle(()=>socket.deliver(STARTED));const reading=(stop:string)=>({schema_version:1,source:{id:"orders",uri:"hick:///orders.md",revision:"captured",bytes:[97]},scope:{session:"dbg-0",run:"run",stop,frame:2,invocation:null},locations:[],annotations:[],unsupported:[]});
  await settle(()=>{socket.deliver({event:"reading",session:"dbg-0",reading:reading("opaque-stop-a")});socket.deliver({...STOPPED,stop:80,thread:0,observation_scope:reading("opaque-stop-a").scope});});expect(result.current.reading?.scope.stop).toBe("opaque-stop-a");
  await settle(()=>socket.deliver({event:"reading",session:"dbg-0",reading:reading("opaque-stop-b")}));expect(result.current.reading?.scope.stop).toBe("opaque-stop-a");await settle(()=>socket.deliver({...STOPPED,stop:81,thread:0,observation_scope:reading("opaque-stop-b").scope}));expect(result.current.reading?.scope.stop).toBe("opaque-stop-b");
  await settle(()=>{socket.deliver({event:"reading",session:"dbg-0",reading:reading("opaque-stop-a")});socket.deliver({...STOPPED,stop:80,thread:0,observation_scope:reading("opaque-stop-a").scope});});expect(result.current.reading?.scope.stop).toBe("opaque-stop-b");expect(result.current.observationScope?.stop).toBe("opaque-stop-b");
 });

});

it('retains source inventory notices from automatic direct startup',async()=>{vi.mocked(withDebugProtocol).mockImplementationOnce(async(_query,effect)=>{await effect({workspace:'fixture-root',notices:['Unreadable input retained as unsupported.']});});const {hook}=open();await settle(()=>hook.result.current.start());expect(hook.result.current.launchNotices).toEqual(['Unreadable input retained as unsupported.']);});

it("preflights direct startup and retires held negotiation before sending",async()=>{
 vi.mocked(withDebugProtocol).mockRejectedValueOnce(new Error("This engine needs protocol 2"));const {socket,hook}=open();await act(async()=>hook.result.current.start());expect(socket.sent).toMatchObject([]);expect(hook.result.current.message).toContain("protocol 2");
 let ready!:()=>void;vi.mocked(withDebugProtocol).mockImplementationOnce((_query,effect)=>new Promise<void>(resolve=>{ready=()=>{void effect({workspace:"fixture-root"});resolve();};}));await settle(()=>hook.result.current.start());await settle(()=>hook.result.current.stop());await act(async()=>ready());expect(socket.sent.some(item=>(item as {op:string}).op==="start")).toBe(false);expect(hook.result.current.status).toBe("idle");
});

it("refuses new startup from a retired source identity while retaining its existing session controls",async()=>{
 const socket=new FakeSocket(),client=new DebugClient(socket.debug()),identity={};let current=false;socket.onDebugFrame(bytes=>client.handleFrame(bytes));
 const hook=renderHook(()=>useDebuggerOver(client,"old.py",[],"new",identity,()=>current));await settle(()=>hook.result.current.start());expect(socket.sent).toHaveLength(0);expect(hook.result.current.message).toContain("no longer owned");
 current=true;const held=hook.result.current;await settle(()=>{held.start();expect(held.isActive?.()).toBe(true);});await settle(()=>{socket.deliver({...STARTED,doc:"hick:///old.py"});socket.deliver(STOPPED);});const frames=hook.result.current.frames,currentCount=socket.sent.length;current=false;
 await settle(()=>hook.result.current.start());expect(socket.sent).toHaveLength(currentCount);expect(hook.result.current.status).toBe("paused");expect(hook.result.current.frames).toBe(frames);await settle(()=>hook.result.current.stop());expect(socket.sent.at(-1)).toMatchObject({op:"stop",session:"dbg-0"});
});
// These controller witnesses use a provider-owned source capability. Legacy
// wire fixtures above retain their prior protocol assumptions until migrated.
describe('guarded protocol3 controller',()=>{
 async function fixture(){
  const moduleName='node:'+'crypto';const {webcrypto}=await import(moduleName);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});
  const socket=new FakeSocket(),identity={},state={text:Array.from({length:100},(_,index)=>`line${index}`).join('\n'),current:true};
  const sources={read:(uri:string)=>{const text=state.text;return {uri,workspace:'guarded-root',namespace:'guarded-root',identity,text,saved:true,current:()=>state.current&&state.text===text};}};
  const client=new DebugClient({...socket.debug(),provider:{protocol:3,workspace:'guarded-root',sources}});socket.onDebugFrame(frame=>client.handleFrame(frame));
  const hook=renderHook(()=>useDebuggerOver(client,'orders.md',[],'new',identity,()=>state.current));
  async function launch(){await act(async()=>{hook.result.current.start();});await waitFor(()=>expect(socket.sent.some(item=>(item as {op:string}).op==='start')).toBe(true));const request=socket.sent.find(item=>(item as {op:string}).op==='start') as {request_id:string;source_hash:string};await settle(()=>socket.deliver({...STARTED,request_id:request.request_id,source_hash:request.source_hash,breakpoints:[]}));await settle(()=>socket.deliver({...STOPPED,stop:1,thread:0,frames:STOPPED.frames.map(frame=>({...frame,source_hash:request.source_hash}))}));return request;}
  return {socket,hook,state,launch};
 }
 it('keeps unconfirmed effects owned and rejects old pause resurrection through resource readdress',async()=>{
  const {socket,hook,state,launch}=await fixture();await launch();expect(hook.result.current.status).toBe('paused');
  await settle(()=>hook.result.current.step('over'));const movement=socket.sent.at(-1) as {request_id:string};expect(movement).toMatchObject({op:'step',stop:1,thread:0,frame:2});
  await settle(()=>socket.deliver({event:'failed',session:'dbg-0',about:'step',request_id:movement.request_id,scope_retired:true,message:'stop observation unavailable'}));expect(hook.result.current.status).toBe('unconfirmed');expect(hook.result.current.isActive?.()).toBe(true);expect(hook.result.current.historicalFrames).toEqual(STOPPED.frames.map(frame=>({...frame,source_hash:expect.any(String)})));
  state.current=false;const count=socket.sent.length;await settle(()=>{hook.result.current.start();hook.result.current.step('continue');hook.result.current.jumpTo(3);});expect(socket.sent).toHaveLength(count);expect(await hook.result.current.query('quantity')).toBeNull();
  await settle(()=>socket.deliver({...STOPPED,stop:1,thread:0}));expect(hook.result.current.status).toBe('unconfirmed');expect(hook.result.current.captureFrame?.(2)).toBeNull();await settle(()=>hook.result.current.stop());expect(socket.sent.at(-1)).toMatchObject({op:'stop',session:'dbg-0'});expect(hook.result.current.isActive?.()).toBe(true);await settle(()=>socket.deliver({event:'ended',session:'dbg-0'}));expect(hook.result.current.isActive?.()).toBe(false);
 });
 it('restores only a correlated pre-effect pause and rejects malformed stop contents before advancing generation',async()=>{
  const {socket,hook,launch}=await fixture();await launch();await settle(()=>hook.result.current.step('over'));const movement=socket.sent.at(-1) as {request_id:string};await settle(()=>socket.deliver({event:'failed',session:'dbg-0',about:'step',request_id:'foreign',scope_retired:false,message:'foreign'}));expect(hook.result.current.status).toBe('running');await settle(()=>socket.deliver({event:'failed',session:'dbg-0',about:'step',request_id:movement.request_id,scope_retired:false,message:'not admitted'}));expect(hook.result.current.status).toBe('paused');expect(hook.result.current.selectedFrame).toBe(2);
  await settle(()=>socket.deliver({...STOPPED,stop:99,thread:0,frames:[{id:true,name:'bad'}]}));expect(hook.result.current.selectedFrame).toBe(2);await settle(()=>socket.deliver({...STOPPED,stop:2,thread:0}));expect(hook.result.current.status).toBe('paused');expect(hook.result.current.captureFrame?.(2)?.current()).toBe(true);
 });
 it('does not let a late recursive reply survive closing the same-frame inspection',async()=>{
  const {socket,hook,launch}=await fixture();await launch();await settle(()=>socket.deliver({...STOPPED,stop:2,thread:0,variables:[{name:'object',value:'reported',type_name:'dict',variables_reference:7}]}));const root=hook.result.current.variableRoots![0];await settle(()=>hook.result.current.expandValue?.(root.id));const request=socket.sent.at(-1) as {request_id:string};expect(request).toMatchObject({op:'children',frame:2,stop:2,thread:0,reference:7});await settle(()=>hook.result.current.closeValues?.());await settle(()=>socket.deliver({event:'children',session:'dbg-0',frame:2,stop:2,thread:0,reference:7,request_id:request.request_id,variables:[{name:'late',value:'not current',variables_reference:0}]}));expect(hook.result.current.valueNode?.(root.id)).toBeNull();expect(hook.result.current.variableRoots?.[0].children).toBeNull();
 });
});

import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { Realtime } from "../api/realtime";
import { useDebugger } from "./useDebugger";

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
  breakpoints: [{ line: 25, verified: true }],
};

const STOPPED = {
  event: "stopped",
  session: "dbg-0",
  reason: "breakpoint",
  line: 14,
  frames: [{ id: 2, name: "line_total", line: 14, source: "orders.py", in_document: true }],
  variables: [{ name: "quantity", value: "2", type_name: "int", variables_reference: 0 }],
};

function open() {
  const socket = new FakeSocket();
  const hook = renderHook(() => useDebugger(socket as unknown as Realtime, "orders.hick"));
  return { socket, hook };
}

describe("the debugger, over the socket", () => {
  it("leaves 'starting' when the engine answers", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    expect(hook.result.current.status).toBe("starting");
    expect(socket.sent[0]).toMatchObject({ op: "start", doc: "hick:///orders.hick" });

    act(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));
    expect(hook.result.current.capabilities?.goto_targets).toBe(true);
  });

  it("shows the paused line and the frame's values", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver(STOPPED));

    await waitFor(() => expect(hook.result.current.status).toBe("paused"));
    expect(hook.result.current.pausedLine).toBe(14);
    expect(hook.result.current.frames[0].name).toBe("line_total");
    expect(hook.result.current.variables[0].value).toBe("2");
    // The top frame is selected, so evaluating goes to the right one.
    expect(hook.result.current.selectedFrame).toBe(2);
  });

  it("sends the session id with every later request", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver(STOPPED));

    act(() => hook.result.current.step("over"));
    act(() => hook.result.current.evaluate("quantity * 2"));
    act(() => hook.result.current.jumpTo(12));

    expect(socket.sent.slice(1)).toEqual([
      { op: "step", session: "dbg-0", how: "over", frame: 2 },
      { op: "eval", session: "dbg-0", expression: "quantity * 2", context: "repl", frame: 2 },
      { op: "jump", session: "dbg-0", line: 12 },
    ]);
  });

  it("does nothing before a session exists", () => {
    // Stepping with no session would be a request the engine answers with an
    // error, and the error would be the UI's fault.
    const { socket, hook } = open();
    act(() => hook.result.current.step("over"));
    act(() => hook.result.current.evaluate("x"));
    expect(socket.sent).toEqual([]);
  });

  it("keeps breakpoints across a session, and sends them when one starts", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.toggleBreakpoint(14));
    act(() => hook.result.current.start());
    expect(socket.sent.at(-1)).toMatchObject({ breakpoints: [{ line: 14 }] });

    act(() => socket.deliver(STARTED));
    await waitFor(() => expect(hook.result.current.status).toBe("running"));
    // The engine's answer replaces the optimistic set: it knows which bound.
    expect(hook.result.current.breakpoints).toEqual([{ line: 25, verified: true }]);
  });

  it("reports a failure instead of waiting forever", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() =>
      socket.deliver({
        event: "failed",
        message: "no debug adapter for python on this machine.",
      }),
    );
    await waitFor(() => expect(hook.result.current.status).toBe("failed"));
    expect(hook.result.current.message).toContain("no debug adapter");
  });

  it("goes back to idle when the session ends", async () => {
    const { socket, hook } = open();
    act(() => hook.result.current.start());
    act(() => socket.deliver(STARTED));
    act(() => socket.deliver(STOPPED));
    act(() => socket.deliver({ event: "ended", session: "dbg-0" }));

    await waitFor(() => expect(hook.result.current.status).toBe("idle"));
    expect(hook.result.current.pausedLine).toBeNull();
    expect(hook.result.current.frames).toEqual([]);
  });
});

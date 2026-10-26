<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/terminal/useTerminals.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-381dcfbbd7e7"></a>
## `apps/web/src/terminal/useTerminals.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-381dcfbbd7e7-POLL_MS"></a>
### `POLL_MS`

The window's terminal sessions, kept fresh.
One poll for the whole window rather than one per pane: the attention queue
is a property of every session at once, including the ones whose panes are
closed, so there is nothing for a per-pane subscription to be a
subscription TO. The bytes are the part that needs a socket, and those have
one (see TerminalPane) — this is only the supervision state.
Polling, not a push channel, and deliberately: two of the five states are
decided by a clock (a session goes idle by being quiet), so something has
to re-ask on a timer whatever the transport is. A second is well under the
time it takes to look away and back.

<storycode:copy id="f-381dcfbbd7e7-POLL_MS">
// The window's terminal sessions, kept fresh.
//
// One poll for the whole window rather than one per pane: the attention queue
// is a property of every session at once, including the ones whose panes are
// closed, so there is nothing for a per-pane subscription to be a
// subscription TO. The bytes are the part that needs a socket, and those have
// one (see TerminalPane) — this is only the supervision state.
//
// Polling, not a push channel, and deliberately: two of the five states are
// decided by a clock (a session goes idle by being quiet), so something has
// to re-ask on a timer whatever the transport is. A second is well under the
// time it takes to look away and back.

import { useCallback, useEffect, useRef, useState } from "react";

import { api } from "../api/client";
import { useTerminalActions } from "./uiTerminalActions";
import type { OpenTerminal, TerminalAnchor, TerminalSession } from "../api/types";

export const POLL_MS = 1000;

</storycode:copy>
<a id="f-381dcfbbd7e7-Terminals"></a>
### `Terminals`

<storycode:copy id="f-381dcfbbd7e7-Terminals">
export interface Terminals {
  sessions: TerminalSession[];
  /** Ids claiming attention, most-claiming first. The server's order. */
  attention: string[];
  /** Which terminals are writing into which documents, by session id.
   * Polled with everything else rather than pushed: "never anchor silently"
   * is a claim about what is on screen at any moment, and a state that is
   * only announced when it changes is one a reconnect can lose. */
  anchors: Record<string, TerminalAnchor>;
  turbo: boolean;
  error: string | null;
  open: (spec?: OpenTerminal) => Promise<TerminalSession | null>;
  close: (id: string) => Promise<boolean>;
  answer: (id: string, send: string) => Promise<boolean>;
  interrupt: (id: string) => Promise<boolean>;
  setTurbo: (enabled: boolean) => Promise<boolean>;
  /** Bind a terminal to a container in a document, and start recording. */
  anchor: (id: string, doc: string, container: string) => Promise<boolean>;
  unanchor: (id: string) => Promise<boolean>;
  /** Record again after a suspension. Always a new cell. */
  resumeAnchor: (id: string) => Promise<boolean>;
  refresh: () => void;
}

</storycode:copy>
<a id="f-381dcfbbd7e7-useTerminals"></a>
### `useTerminals`

<storycode:copy id="f-381dcfbbd7e7-useTerminals">
export function useTerminals(): Terminals {
  const [sessions, setSessions] = useState<TerminalSession[]>([]);
  const [attention, setAttention] = useState<string[]>([]);
  const [turbo, setTurboState] = useState(false);
  const [anchors, setAnchors] = useState<Record<string, TerminalAnchor>>({});
  const [readError, setReadError] = useState<string | null>(null);
  const [operationError, setOperationError] = useState<string | null>(null);
  const life = useRef({ live: false, sequence: 0 });
  const refresh = useCallback(() => {
    const captured = life.current, ticket = ++captured.sequence;
    if (!captured.live) return;
    const current = () => captured.live && captured.sequence === ticket;
    void api.terminalAnchors().then(state => { if (current()) setAnchors(state.anchors); }, () => {});
    void api.terminals().then(state => {
      if (!current()) return;
      setSessions(state.sessions); setAttention(state.attention); setTurboState(state.turbo); setReadError(null);
    }, cause => { if (current()) setReadError(String(cause)); });
  }, []);
  useEffect(() => {
    const captured = { live: true, sequence: 0 }; life.current = captured;
    refresh();
    const timer = window.setInterval(refresh, POLL_MS);
    return () => { captured.live = false; captured.sequence++; window.clearInterval(timer); };
  }, [refresh]);
  const perform = useCallback(async (what: () => Promise<unknown>): Promise<boolean> => {
    const captured = life.current;
    if (!captured.live) return false;
    try {
      await what();
      if (captured.live) { setOperationError(null); refresh(); }
      return true;
    } catch (cause) {
      if (captured.live) { setOperationError(`${String(cause)}. Inspect terminal state before retrying; effects are not rolled back.`); refresh(); }
      return false;
    }
  }, [refresh]);
  const open = useCallback(async (spec: OpenTerminal = {}) => {
    let session: TerminalSession | null = null;
    await perform(async () => { session = await api.openTerminal(spec); });
    return session;
  }, [perform]);
  const close = useCallback((id: string) => perform(() => api.closeTerminal(id)), [perform]);
  const answer = useCallback((id: string, send: string) => perform(() => api.answerTerminal(id, send)), [perform]);
  const interrupt = useCallback((id: string) => perform(() => api.interruptTerminal(id)), [perform]);
  const setTurbo = useCallback((enabled: boolean) => perform(() => api.setTurbo(enabled)), [perform]);
  const anchor = useCallback((id: string, doc: string, container: string) => perform(() => api.anchorTerminal(id, doc, container)), [perform]);
  const unanchor = useCallback((id: string) => perform(() => api.unanchorTerminal(id)), [perform]);
  const resumeAnchor = useCallback((id: string) => perform(() => api.resumeAnchor(id)), [perform]);
  const error = operationError ?? readError;

  const controller = {
    sessions,
    attention,
    anchors,
    turbo,
    error,
    open,
    close,
    answer,
    interrupt,
    setTurbo,
    anchor,
    unanchor,
    resumeAnchor,
    refresh,
  };
  useTerminalActions(controller);
  return controller;
}

</storycode:copy>
<a id="f-381dcfbbd7e7-sessionById"></a>
### `sessionById`

<storycode:copy id="f-381dcfbbd7e7-sessionById">
/** A session by id, or null. */
export function sessionById(
  sessions: readonly TerminalSession[],
  id: string | null,
): TerminalSession | null {
  if (id === null) return null;
  return sessions.find((s) => s.id === id) ?? null;
}
</storycode:copy>

### Assemble `apps/web/src/terminal/useTerminals.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/terminal/useTerminals.ts"><storycode:paste select="#f-381dcfbbd7e7-POLL_MS"/><storycode:paste select="#f-381dcfbbd7e7-Terminals"/><storycode:paste select="#f-381dcfbbd7e7-useTerminals"/><storycode:paste select="#f-381dcfbbd7e7-sessionById"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-bcb81aafb218"></a>
## `apps/web/src/terminal/useTerminals.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-bcb81aafb218-response"></a>
### `response`

<storycode:copy id="f-bcb81aafb218-response">
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { useTerminals } from "./useTerminals";
import type { TerminalSession } from "../api/types";
const response = (sessions: TerminalSession[] = []) => ({ sessions, attention: [], turbo: false });
</storycode:copy>
<a id="f-bcb81aafb218-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-bcb81aafb218-expression_statement">
beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(api, "terminals").mockResolvedValue(response());
  vi.spyOn(api, "terminalAnchors").mockResolvedValue({ anchors: {} });
});
</storycode:copy>
<a id="f-bcb81aafb218-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-bcb81aafb218-expression_statement-2">
afterEach(cleanup);
</storycode:copy>
<a id="f-bcb81aafb218-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-bcb81aafb218-expression_statement-3">
it("reports actual terminal refusals and keeps them visible across successful supervision refreshes", async () => {
  const close = vi.spyOn(api, "closeTerminal").mockRejectedValueOnce(Error("actual close refusal")).mockResolvedValueOnce(undefined);
  const hook = renderHook(() => useTerminals());
  await act(async () => { expect(await hook.result.current.close("one")).toBe(false); });
  await act(async () => { hook.result.current.refresh(); });
  expect(hook.result.current.error).toContain("actual close refusal");
  await act(async () => { expect(await hook.result.current.close("one")).toBe(true); });
  expect(hook.result.current.error).toBeNull();
  expect(close).toHaveBeenCalledTimes(2);
});
</storycode:copy>
<a id="f-bcb81aafb218-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-bcb81aafb218-expression_statement-4">
it("refuses stale supervision replies and closed-controller mutations", async () => {
  let finishOld!: (value: ReturnType<typeof response>) => void;
  vi.mocked(api.terminals).mockImplementationOnce(() => new Promise(resolve => { finishOld = resolve; })).mockResolvedValue(response());
  const close = vi.spyOn(api, "closeTerminal");
  const hook = renderHook(() => useTerminals());
  await act(async () => { hook.result.current.refresh(); });
  await act(async () => { finishOld({ ...response(), turbo: true }); });
  expect(hook.result.current.turbo).toBe(false);
  const cachedClose = hook.result.current.close;
  hook.unmount();
  expect(await cachedClose("one")).toBe(false);
  expect(close).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-bcb81aafb218-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-bcb81aafb218-expression_statement-5">
it("does not optimistically enable automatic answers when the host refuses", async () => {
  vi.spyOn(api, "setTurbo").mockRejectedValue(Error("host policy refusal"));
  const hook = renderHook(() => useTerminals());
  await act(async () => { expect(await hook.result.current.setTurbo(true)).toBe(false); });
  expect(hook.result.current.turbo).toBe(false);
  await waitFor(() => expect(hook.result.current.error).toContain("host policy refusal"));
});
</storycode:copy>

### Assemble `apps/web/src/terminal/useTerminals.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/terminal/useTerminals.test.tsx"><storycode:paste select="#f-bcb81aafb218-response"/><storycode:paste select="#f-bcb81aafb218-expression_statement"/><storycode:paste select="#f-bcb81aafb218-expression_statement-2"/><storycode:paste select="#f-bcb81aafb218-expression_statement-3"/><storycode:paste select="#f-bcb81aafb218-expression_statement-4"/><storycode:paste select="#f-bcb81aafb218-expression_statement-5"/></storycode:file>


</storycode:doc>

<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/terminal/useTerminals.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `73a3e25bbf4c944a94986b53039f970577e91efed0cf46e7cda796ac322591e4`.

<book:entry path="apps/web/src/terminal/useTerminals.ts">
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

/** A session by id, or null. */
export function sessionById(
  sessions: readonly TerminalSession[],
  id: string | null,
): TerminalSession | null {
  if (id === null) return null;
  return sessions.find((s) => s.id === id) ?? null;
}
</book:entry>

</book:doc>

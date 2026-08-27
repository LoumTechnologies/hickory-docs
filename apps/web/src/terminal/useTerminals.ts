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
  close: (id: string) => Promise<void>;
  answer: (id: string, send: string) => Promise<void>;
  interrupt: (id: string) => Promise<void>;
  setTurbo: (enabled: boolean) => Promise<void>;
  /** Bind a terminal to a container in a document, and start recording. */
  anchor: (id: string, doc: string, container: string) => Promise<void>;
  unanchor: (id: string) => Promise<void>;
  /** Record again after a suspension. Always a new cell. */
  resumeAnchor: (id: string) => Promise<void>;
  refresh: () => void;
}

export function useTerminals(): Terminals {
  const [sessions, setSessions] = useState<TerminalSession[]>([]);
  const [attention, setAttention] = useState<string[]>([]);
  const [turbo, setTurboState] = useState(false);
  const [anchors, setAnchors] = useState<Record<string, TerminalAnchor>>({});
  const [error, setError] = useState<string | null>(null);
  const live = useRef(true);

  const refresh = useCallback(() => {
    // Its own request rather than a field on `/terminals`: an anchor is a
    // fact about a session AND a document, and the terminal list belongs to
    // sessions alone. A failure here leaves the sessions readable.
    api.terminalAnchors().then(
      (state) => {
        if (live.current) setAnchors(state.anchors);
      },
      () => {},
    );
    api.terminals().then(
      (state) => {
        if (!live.current) return;
        setSessions(state.sessions);
        setAttention(state.attention);
        setTurboState(state.turbo);
        setError(null);
      },
      (e) => {
        if (!live.current) return;
        setError(e instanceof Error ? e.message : String(e));
      },
    );
  }, []);

  useEffect(() => {
    live.current = true;
    refresh();
    const timer = window.setInterval(refresh, POLL_MS);
    return () => {
      live.current = false;
      window.clearInterval(timer);
    };
  }, [refresh]);

  const open = useCallback(
    async (spec: OpenTerminal = {}) => {
      try {
        const session = await api.openTerminal(spec);
        refresh();
        return session;
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        return null;
      }
    },
    [refresh],
  );

  const close = useCallback(
    async (id: string) => {
      // A close that races another close is not an error worth showing: the
      // session is gone either way, which is what was asked for.
      await api.closeTerminal(id).catch(() => {});
      refresh();
    },
    [refresh],
  );

  const answer = useCallback(
    async (id: string, send: string) => {
      try {
        await api.answerTerminal(id, send);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
      refresh();
    },
    [refresh],
  );

  const interrupt = useCallback(
    async (id: string) => {
      await api.interruptTerminal(id).catch(() => {});
      refresh();
    },
    [refresh],
  );

  const setTurbo = useCallback(
    async (enabled: boolean) => {
      // Optimistic, because the switch should feel like a switch; the next
      // poll is the truth either way.
      setTurboState(enabled);
      await api.setTurbo(enabled).catch(() => {});
      refresh();
    },
    [refresh],
  );

  const anchor = useCallback(
    async (id: string, doc: string, container: string) => {
      try {
        await api.anchorTerminal(id, doc, container);
      } catch (e) {
        // A refusal here is a sentence a person needs to read — an
        // un-hookable shell says which shells work — so it goes where
        // errors go rather than being swallowed.
        setError(e instanceof Error ? e.message : String(e));
      }
      refresh();
    },
    [refresh],
  );

  const unanchor = useCallback(
    async (id: string) => {
      await api.unanchorTerminal(id).catch(() => {});
      refresh();
    },
    [refresh],
  );

  const resumeAnchor = useCallback(
    async (id: string) => {
      try {
        await api.resumeAnchor(id);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
      refresh();
    },
    [refresh],
  );

  return {
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
}

/** A session by id, or null. */
export function sessionById(
  sessions: readonly TerminalSession[],
  id: string | null,
): TerminalSession | null {
  if (id === null) return null;
  return sessions.find((s) => s.id === id) ?? null;
}

// The agent chat dock along the bottom of a document.
//
// The conversation is a TREE, not a transcript. Every message records the turn
// it continued from, so selecting an earlier turn and sending again forks a
// branch rather than destroying what came after it — you can try a different
// instruction from the same starting point and keep both. What the dock shows
// is the path from the root to the currently selected tip; where a turn has
// more than one child, a small switcher moves between the alternatives.
//
// Live output streams on the run channel keyed by the turn id, which is also
// the run id, so the answer appears as it is generated and is then replaced by
// the persisted turn when the session finishes.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api } from "../api/client";
import type { AgentTurn } from "../api/types";
import type { Realtime } from "../api/realtime";

export interface ChatDockProps {
  docId: string;
  realtime: Realtime;
  /** Collapse to the composer only. */
  collapsed: boolean;
  onToggleCollapsed: () => void;
  /** The agent may have edited the document; refresh the views. */
  onAgentFinished: () => void;
}

/** Path from a root turn down to `tip`, oldest first. */
export function branchOf(turns: AgentTurn[], tip: string | null): AgentTurn[] {
  if (!tip) return [];
  const byId = new Map(turns.map((t) => [t.id, t]));
  const path: AgentTurn[] = [];
  let cursor: string | null = tip;
  const seen = new Set<string>();
  while (cursor && !seen.has(cursor)) {
    seen.add(cursor);
    const turn = byId.get(cursor);
    if (!turn) break;
    path.push(turn);
    cursor = turn.parent_id;
  }
  return path.reverse();
}

/** Children of `parent` (null = conversation roots), oldest first. */
export function childrenOf(turns: AgentTurn[], parent: string | null): AgentTurn[] {
  return turns.filter((t) => t.parent_id === parent);
}

/** The newest turn, used as the default tip when nothing is selected. */
function newestTurn(turns: AgentTurn[]): AgentTurn | null {
  return turns.length === 0 ? null : turns[turns.length - 1];
}

/** Follow the newest child at each step — the tip a branch switch lands on. */
export function deepestFrom(turns: AgentTurn[], from: string): string {
  let cursor = from;
  for (;;) {
    const kids = childrenOf(turns, cursor);
    if (kids.length === 0) return cursor;
    cursor = kids[kids.length - 1].id;
  }
}

export function ChatDock({
  docId,
  realtime,
  collapsed,
  onToggleCollapsed,
  onAgentFinished,
}: ChatDockProps) {
  const [turns, setTurns] = useState<AgentTurn[]>([]);
  const [tip, setTip] = useState<string | null>(null);
  const [prompt, setPrompt] = useState("");
  const [running, setRunning] = useState<string | null>(null);
  const [stream, setStream] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [unavailable, setUnavailable] = useState(false);

  const logRef = useRef<HTMLDivElement>(null);
  const runningRef = useRef<string | null>(null);
  runningRef.current = running;
  const onFinishedRef = useRef(onAgentFinished);
  onFinishedRef.current = onAgentFinished;

  const refresh = useCallback(
    () =>
      api.agentTurns(docId).then(
        (r) => {
          setTurns(r.turns);
          setTip((current) => current ?? newestTurn(r.turns)?.id ?? null);
        },
        () => undefined,
      ),
    [docId],
  );

  useEffect(() => {
    void refresh();
  }, [refresh]);

  // Live agent output for the turn in flight.
  useEffect(
    () =>
      realtime.onRunEvent((msg) => {
        if (msg.run_id !== runningRef.current) return;
        if ("event" in msg) {
          const e = msg.event;
          if (e.kind === "out" || e.kind === "err") setStream((prev) => prev + e.data);
          return;
        }
        setRunning(null);
        setStream("");
        void refresh();
        onFinishedRef.current();
      }),
    [realtime, refresh],
  );

  const branch = useMemo(() => branchOf(turns, tip), [turns, tip]);

  useEffect(() => {
    const log = logRef.current;
    if (log) log.scrollTop = log.scrollHeight;
  }, [branch.length, stream, collapsed]);

  const send = async () => {
    const text = prompt.trim();
    if (!text || running) return;
    setError(null);
    setStream("");
    try {
      // The visible tip is the parent: rewinding is just selecting an earlier
      // turn before sending.
      const { session_id } = await api.agent(docId, text, tip);
      setRunning(session_id);
      runningRef.current = session_id;
      setPrompt("");
      // Show the new turn immediately rather than waiting for the round trip.
      setTurns((prev) => [
        ...prev,
        {
          id: session_id,
          parent_id: tip,
          prompt: text,
          answer: null,
          status: "running",
          error: null,
          created_at: new Date().toISOString(),
        },
      ]);
      setTip(session_id);
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      // A server without ANTHROPIC_API_KEY says so with 503; that is a
      // configuration fact, not a failure to report as a red error every time.
      if (/agent not configured/i.test(message)) setUnavailable(true);
      else setError(message);
    }
  };

  return (
    <section className={`chat-dock${collapsed ? " collapsed" : ""}`} aria-label="Agent chat">
      <header className="chat-head">
        <button
          className="chat-toggle"
          aria-expanded={!collapsed}
          onClick={onToggleCollapsed}
          title={collapsed ? "Show conversation" : "Hide conversation"}
        >
          <span aria-hidden="true">{collapsed ? "▴" : "▾"}</span> Agent
        </button>
        {turns.length > 0 && (
          <span className="chat-count">
            {branch.length} of {turns.length} turn{turns.length === 1 ? "" : "s"} on this branch
          </span>
        )}
        {tip && (
          <button
            className="btn-link chat-new"
            onClick={() => setTip(null)}
            title="Start a conversation that does not continue from any existing turn"
          >
            New thread
          </button>
        )}
      </header>

      {!collapsed && (
        <div className="chat-log" ref={logRef}>
          {branch.length === 0 && !running && (
            <p className="muted chat-empty">
              Ask the agent to edit, extend, or verify this document. It works through
              the document and its generated output, and every session is saved into
              the project as a <code>hick:session</code> file.
            </p>
          )}
          {branch.map((turn) => {
            const siblings = childrenOf(turns, turn.parent_id);
            const index = siblings.findIndex((s) => s.id === turn.id);
            return (
              <article key={turn.id} className="chat-turn">
                <div className="chat-msg chat-user">
                  <span className="chat-role">you</span>
                  <p>{turn.prompt}</p>
                  <div className="chat-turn-actions">
                    {siblings.length > 1 && (
                      <span className="chat-branch" role="group" aria-label="Branch">
                        <button
                          className="btn-link"
                          disabled={index <= 0}
                          aria-label="Previous branch"
                          onClick={() => setTip(deepestFrom(turns, siblings[index - 1].id))}
                        >
                          ‹
                        </button>
                        {index + 1}/{siblings.length}
                        <button
                          className="btn-link"
                          disabled={index >= siblings.length - 1}
                          aria-label="Next branch"
                          onClick={() => setTip(deepestFrom(turns, siblings[index + 1].id))}
                        >
                          ›
                        </button>
                      </span>
                    )}
                    {turn.id !== tip && (
                      <button
                        className="btn-link chat-rewind"
                        onClick={() => setTip(turn.id)}
                        title="Continue from here — later turns stay on their own branch"
                      >
                        rewind here
                      </button>
                    )}
                  </div>
                </div>
                <div className="chat-msg chat-agent">
                  <span className="chat-role">agent</span>
                  {turn.id === running ? (
                    <pre className="chat-stream">
                      {stream}
                      <span className="t-cursor">▋</span>
                    </pre>
                  ) : turn.status === "error" ? (
                    <p className="chat-error">{turn.error ?? "session failed"}</p>
                  ) : turn.answer ? (
                    <p>{turn.answer}</p>
                  ) : (
                    <p className="muted">no answer recorded</p>
                  )}
                </div>
              </article>
            );
          })}
        </div>
      )}

      <div className="chat-composer">
        <textarea
          value={prompt}
          rows={collapsed ? 1 : 2}
          placeholder={
            tip ? "Reply, or rewind to an earlier turn to branch…" : "Ask the agent…"
          }
          onChange={(e) => setPrompt(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              void send();
            }
          }}
        />
        <button
          className="btn btn-primary"
          disabled={!prompt.trim() || running !== null}
          onClick={() => void send()}
        >
          {running ? "Working…" : "Send"}
        </button>
      </div>
      {unavailable && (
        <p className="chat-note muted">
          The agent is not configured on this server (no <code>ANTHROPIC_API_KEY</code>).
        </p>
      )}
      {error && <p className="chat-note error">{error}</p>}
    </section>
  );
}

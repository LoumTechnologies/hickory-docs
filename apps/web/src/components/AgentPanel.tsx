import { useEffect, useRef, useState } from "react";
import { api } from "../api/client";
import type { Realtime } from "../api/realtime";

interface AgentEvent {
  kind: "out" | "err" | "status";
  text: string;
}

/** Prompt box that starts an agent session and streams its events in. */
export function AgentPanel({ docId, realtime }: { docId: string; realtime: Realtime }) {
  const [prompt, setPrompt] = useState("");
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [events, setEvents] = useState<AgentEvent[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const logRef = useRef<HTMLPreElement>(null);
  const sessionRef = useRef<string | null>(null);
  sessionRef.current = sessionId;

  useEffect(() => {
    return realtime.onRunEvent((msg) => {
      if (msg.run_id !== sessionRef.current) return;
      if ("event" in msg) {
        const e = msg.event;
        if (e.kind === "out" || e.kind === "err") {
          setEvents((prev) => [...prev, { kind: e.kind as "out" | "err", text: e.data }]);
        }
      } else {
        setBusy(false);
        setEvents((prev) => [...prev, { kind: "status", text: `session ${msg.status}` }]);
      }
    });
  }, [realtime]);

  useEffect(() => {
    const log = logRef.current;
    if (log) log.scrollTop = log.scrollHeight;
  }, [events]);

  const start = async () => {
    const p = prompt.trim();
    if (!p || busy) return;
    setError(null);
    setBusy(true);
    setEvents([]);
    try {
      const { session_id } = await api.agent(docId, p);
      setSessionId(session_id);
      sessionRef.current = session_id;
    } catch (err) {
      setBusy(false);
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  return (
    <aside className="agent-panel">
      <h2>Agent</h2>
      <p className="agent-hint">
        Sessions are persisted as <code>hick:session</code> documents in the project repo.
      </p>
      <div className="agent-input">
        <textarea
          value={prompt}
          placeholder="Ask the agent to edit, verify, or extend this document…"
          rows={3}
          onChange={(e) => setPrompt(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) void start();
          }}
        />
        <button className="btn btn-primary" disabled={busy || !prompt.trim()} onClick={() => void start()}>
          {busy ? "Working…" : "Start session"}
        </button>
      </div>
      {error && <p className="error">{error}</p>}
      {(events.length > 0 || busy) && (
        <pre ref={logRef} className="terminal agent-log">
          {events.map((e, i) => (
            <span key={i} className={e.kind === "err" ? "t-err" : e.kind === "status" ? "t-cmd" : "t-out"}>
              {e.text}
              {e.kind === "status" ? "\n" : ""}
            </span>
          ))}
          {busy && <span className="t-cursor">▋</span>}
        </pre>
      )}
    </aside>
  );
}

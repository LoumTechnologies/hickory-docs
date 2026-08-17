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
import type { AgentTotals, AgentTurn, AgentWsEvent, TranscriptEvent } from "../api/types";
import type { Realtime } from "../api/realtime";

/** The providers the backend accepts (ProviderSelection::ALL), with each
 * one's default model as the free-text input's placeholder. */
export const PROVIDERS: { id: string; label: string; defaultModel: string }[] = [
  { id: "anthropic", label: "Anthropic", defaultModel: "claude-sonnet-5" },
  { id: "openai", label: "OpenAI", defaultModel: "gpt-5" },
  { id: "deepseek", label: "DeepSeek", defaultModel: "deepseek-chat" },
  { id: "grok", label: "xAI (Grok)", defaultModel: "grok-4" },
  { id: "openrouter", label: "OpenRouter", defaultModel: "openrouter/auto" },
];

/** The model a provider runs when none is typed, for the placeholder. */
export function defaultModelFor(provider: string): string {
  return PROVIDERS.find((p) => p.id === provider)?.defaultModel ?? "";
}

/** Compact token count: 950 → "950", 12,400 → "12.4k", 3,100,000 → "3.1M". */
export function formatTokens(n: number): string {
  const compact = (value: number, suffix: string) => {
    const rounded = value >= 100 ? Math.round(value).toString() : value.toFixed(1).replace(/\.0$/, "");
    return `${rounded}${suffix}`;
  };
  if (n < 1000) return String(n);
  if (n < 1_000_000) return compact(n / 1000, "k");
  return compact(n / 1_000_000, "M");
}

/** Session cost to four decimal places; "$—" when a model's price is
 * unknown (the server reports unknown, never a guess). */
export function formatUsd(usd: number | null): string {
  return usd === null ? "$—" : `$${usd.toFixed(4)}`;
}

/** Cache hit rate = reads / (uncached input + reads); null before any
 * input-bearing turn, so the dock can show "—" instead of a fake 0%. */
export function cacheHitRate(input: number, cacheRead: number): number | null {
  const denominator = input + cacheRead;
  return denominator === 0 ? null : cacheRead / denominator;
}

/** The dock header's one-line spend summary,
 * e.g. "$0.0342 · in 12.4k · out 3.1k · cache 78%". */
export function statsLine(t: AgentTotals): string {
  const rate = cacheHitRate(t.input, t.cache_read);
  const cache = rate === null ? "cache —" : `cache ${Math.round(rate * 100)}%`;
  return [formatUsd(t.usd), `in ${formatTokens(t.input)}`, `out ${formatTokens(t.output)}`, cache].join(" · ");
}

/** The tooltip behind the stats line: all four counters, uncompacted. */
export function statsTooltip(t: AgentTotals): string {
  const n = (v: number) => v.toLocaleString("en-US");
  return [
    `cost ${formatUsd(t.usd)}`,
    `input ${n(t.input)} tokens`,
    `output ${n(t.output)} tokens`,
    `cache read ${n(t.cache_read)} tokens`,
    `cache write ${n(t.cache_write)} tokens`,
  ].join("\n");
}

/** Whether any turn has reported usage yet — before that the stats line
 * would be all zeros, which reads as free rather than unstarted. */
export function hasUsage(t: AgentTotals): boolean {
  return t.input + t.output + t.cache_read + t.cache_write > 0;
}

export interface ChatDockProps {
  docId: string;
  realtime: Realtime;
  /** Collapse to the composer only. */
  collapsed: boolean;
  onToggleCollapsed: () => void;
  /** The agent may have edited the document; refresh the views. */
  onAgentFinished: () => void;
}

/** Fold one live agent event into the streaming preview.
 *
 * Tokens append as they arrive; a script or tool start becomes a one-line
 * marker so the pause while it runs reads as work, not a stall. Everything
 * else (usage, thinking, completions) is bookkeeping the finished turn will
 * render properly, so it adds nothing here. */
export function appendStream(prev: string, e: TranscriptEvent | AgentWsEvent): string {
  if (e.kind === "token" || e.kind === "out" || e.kind === "err") return prev + e.data;
  if (e.kind === "script_started") return `${prev}\n\n[running ${e.lang} script…]\n`;
  if (e.kind === "tool_started") return `${prev}\n\n[${e.name}…]\n`;
  return prev;
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
  const [totals, setTotals] = useState<AgentTotals | null>(null);
  // The model control. Provider is hydrated once from the server's resolved
  // default; the model input stays empty (placeholder = the provider's
  // default) unless the session already chose one explicitly.
  const [provider, setProvider] = useState("");
  const [model, setModel] = useState("");

  const logRef = useRef<HTMLDivElement>(null);
  const runningRef = useRef<string | null>(null);
  runningRef.current = running;
  const onFinishedRef = useRef(onAgentFinished);
  onFinishedRef.current = onAgentFinished;
  const hydratedRef = useRef(false);

  const refresh = useCallback(
    () =>
      api.agentTurns(docId).then(
        (r) => {
          setTurns(r.turns);
          setTip((current) => current ?? newestTurn(r.turns)?.id ?? null);
          setTotals(r.totals);
          // Hydrate the controls once — later polls must not clobber a
          // choice being made in the select/input mid-conversation.
          if (!hydratedRef.current) {
            hydratedRef.current = true;
            setProvider(r.provider);
            if (r.model !== defaultModelFor(r.provider)) setModel(r.model);
          }
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
        // The up-loop's files_changed notice rides the same channel but
        // belongs to the document session, not the agent stream.
        if (!("run_id" in msg) || msg.run_id !== runningRef.current) return;
        if ("event" in msg) {
          const e = msg.event;
          setStream((prev) => appendStream(prev, e));
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
      // turn before sending. The model control rides along exactly as shown:
      // an emptied model input clears back to the provider's default.
      const { session_id } = await api.agent(
        docId,
        text,
        tip,
        provider || undefined,
        provider ? model.trim() : undefined,
      );
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
          provider: provider || "anthropic",
          model: model.trim() || defaultModelFor(provider) || "claude-sonnet-5",
          usage: null,
        },
      ]);
      setTip(session_id);
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      // The local server answers the agent route with a 503 whose message
      // starts "agent not available" when no provider key is in the
      // environment (serve/agent.rs::start_turn); that is a configuration
      // fact, not a failure to report as a red error.
      if (/agent not (configured|available)/i.test(message)) setUnavailable(true);
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
        {totals && hasUsage(totals) && (
          <span className="chat-stats muted" title={statsTooltip(totals)}>
            {statsLine(totals)}
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
        <span className="chat-model" role="group" aria-label="Model choice">
          <select
            aria-label="Provider"
            value={provider}
            disabled={running !== null}
            onChange={(e) => setProvider(e.target.value)}
          >
            {provider === "" && <option value="">provider…</option>}
            {provider !== "" && !PROVIDERS.some((p) => p.id === provider) && (
              <option value={provider}>{provider}</option>
            )}
            {PROVIDERS.map((p) => (
              <option key={p.id} value={p.id}>
                {p.label}
              </option>
            ))}
          </select>
          <input
            aria-label="Model"
            value={model}
            disabled={running !== null}
            placeholder={defaultModelFor(provider) || "default model"}
            title="Model id for the next turn — leave empty for the provider's default"
            onChange={(e) => setModel(e.target.value)}
          />
        </span>
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
          The agent needs a provider API key. Set <code>ANTHROPIC_API_KEY</code> (or{" "}
          <code>OPENAI_API_KEY</code>, <code>DEEPSEEK_API_KEY</code>,{" "}
          <code>XAI_API_KEY</code>) in your environment and reopen this folder.
        </p>
      )}
      {error && <p className="chat-note error">{error}</p>}
    </section>
  );
}

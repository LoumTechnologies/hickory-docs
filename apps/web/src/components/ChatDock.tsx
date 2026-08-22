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
import type {
  AgentTotals,
  AgentTurn,
  AgentWsEvent,
  SessionStep,
  SessionTurn,
  TranscriptEvent,
} from "../api/types";
import type { Realtime } from "../api/realtime";
import { TurnCard } from "./SessionTurns";
import { ChatTree } from "./ChatTree";

/** The providers the backend accepts (ProviderSelection::ALL), with each
 * one's default model as the free-text input's placeholder. */
export const PROVIDERS: { id: string; label: string; defaultModel: string }[] =
  [
    { id: "anthropic", label: "Anthropic", defaultModel: "claude-sonnet-5" },
    { id: "openai", label: "OpenAI", defaultModel: "gpt-5" },
    { id: "deepseek", label: "DeepSeek", defaultModel: "deepseek-chat" },
    { id: "grok", label: "xAI (Grok)", defaultModel: "grok-4" },
    { id: "openrouter", label: "OpenRouter", defaultModel: "openrouter/auto" },
    { id: "gab", label: "Gab AI", defaultModel: "arya" },
  ];

/** The model a provider runs when none is typed, for the placeholder. */
export function defaultModelFor(provider: string): string {
  return PROVIDERS.find((p) => p.id === provider)?.defaultModel ?? "";
}

/** Compact token count: 950 → "950", 12,400 → "12.4k", 3,100,000 → "3.1M". */
export function formatTokens(n: number): string {
  const compact = (value: number, suffix: string) => {
    const rounded =
      value >= 100
        ? Math.round(value).toString()
        : value.toFixed(1).replace(/\.0$/, "");
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
  return [
    formatUsd(t.usd),
    `in ${formatTokens(t.input)}`,
    `out ${formatTokens(t.output)}`,
    cache,
  ].join(" · ");
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
  /** Collapse, for the one place this is still a dock. As a PANE there is
   * nothing to collapse — the pane itself is the affordance, and it can be
   * resized, moved, or closed like anything else. */
  collapsed?: boolean;
  onToggleCollapsed?: () => void;
  /** The agent may have edited the document; refresh the views. */
  onAgentFinished: () => void;
  /** Open a session file (the conversation's record) in the app. */
  onOpenSession?: (path: string) => void;
}

/** Fold one live agent event into the streaming preview.
 *
 * Tokens append as they arrive; a script or tool start becomes a one-line
 * marker so the pause while it runs reads as work, not a stall. Everything
 * else (usage, thinking, completions) is bookkeeping the finished turn will
 * render properly, so it adds nothing here. */
export function appendStream(
  prev: string,
  e: TranscriptEvent | AgentWsEvent,
): string {
  if (e.kind === "token" || e.kind === "out" || e.kind === "err")
    return prev + e.data;
  if (e.kind === "script_started")
    return `${prev}\n\n[running ${e.lang} script…]\n`;
  if (e.kind === "tool_started") return `${prev}\n\n[${e.name}…]\n`;
  return prev;
}

/** The model's reasoning, streamed apart from its answer and shown folded. */
export function appendReasoning(
  prev: string,
  e: TranscriptEvent | AgentWsEvent,
): string {
  return e.kind === "reasoning" ? prev + e.data : prev;
}

/**
 * The composer's slash commands — the keyboard way to move around the tree.
 *
 *   /rewind      make the tip's parent the tip (one step back)
 *   /rewind N    go back N turns
 *   /tree        zoom out: the conversation as a tree (toggle)
 *   /new         start a thread that continues from nothing
 *   /help        list these
 *
 * Anything else starting with "/" is a message, not a command — a path or a
 * date at the start of a sentence must not be eaten.
 */
export type SlashCommand =
  | { kind: "rewind"; steps: number }
  | { kind: "tree" }
  | { kind: "new" }
  | { kind: "help" };

export function parseSlash(text: string): SlashCommand | null {
  const m = text.trim().match(/^\/(rewind|tree|new|help)(?:\s+(\d+))?$/i);
  if (!m) return null;
  switch (m[1].toLowerCase()) {
    case "rewind":
      return { kind: "rewind", steps: m[2] ? Math.max(1, Number(m[2])) : 1 };
    case "tree":
      return { kind: "tree" };
    case "new":
      return { kind: "new" };
    default:
      return { kind: "help" };
  }
}

/** The turn `steps` back from `tip` along parent pointers (null = root). */
export function rewindFrom(
  turns: readonly AgentTurn[],
  tip: string | null,
  steps: number,
): string | null {
  const byId = new Map(turns.map((t) => [t.id, t]));
  let cursor = tip;
  for (let i = 0; i < steps && cursor; i++)
    cursor = byId.get(cursor)?.parent_id ?? null;
  return cursor;
}

export const SLASH_HELP =
  "/rewind [N] — back N turns (default 1) · /tree — zoom out to the tree · /new — start a thread · /help";

/** The dock's turn as the shared renderer's shape (steps arrive lazily). */
function asSessionTurn(t: AgentTurn): SessionTurn {
  return {
    id: t.id,
    parent: t.parent_id,
    prompt: t.prompt,
    provider: t.provider,
    model: t.model,
    steps: [],
    answer: t.answer,
    usage: null,
    session_line: 0,
  };
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
export function childrenOf(
  turns: AgentTurn[],
  parent: string | null,
): AgentTurn[] {
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
  collapsed = false,
  onToggleCollapsed,
  onAgentFinished,
  onOpenSession,
}: ChatDockProps) {
  const [turns, setTurns] = useState<AgentTurn[]>([]);
  const [tip, setTip] = useState<string | null>(null);
  const [prompt, setPrompt] = useState("");
  const [running, setRunning] = useState<string | null>(null);
  const [stream, setStream] = useState("");
  const [reasoning, setReasoning] = useState("");
  const [showTree, setShowTree] = useState(false);
  const [note, setNote] = useState<string | null>(null);
  // Session views, by path, fetched when a turn's work is asked for.
  const viewsRef = useRef(new Map<string, Promise<SessionStep[] | null>>());
  const loadWorkFor = useCallback((turn: AgentTurn) => {
    const path = turn.session;
    if (!path) return undefined;
    return async () => {
      const cached = viewsRef.current;
      if (!cached.has(path)) {
        cached.set(
          path,
          api.sessionView(path).then(
            (r) => r.view.turns.find((t) => t.id === turn.id)?.steps ?? [],
            () => null,
          ),
        );
      }
      return (await cached.get(path)) ?? [];
    };
  }, []);
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
          setReasoning((prev) => appendReasoning(prev, e));
          return;
        }
        setRunning(null);
        setStream("");
        setReasoning("");
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
    setNote(null);
    const slash = parseSlash(text);
    if (slash) {
      setPrompt("");
      switch (slash.kind) {
        case "rewind": {
          const target = rewindFrom(turns, tip, slash.steps);
          setTip(target);
          setNote(
            target
              ? `Tip is now "${turns.find((t) => t.id === target)?.prompt.slice(0, 60) ?? target}" — the next message continues from there.`
              : "Tip is the root — the next message starts a new thread.",
          );
          return;
        }
        case "tree":
          setShowTree((v) => !v);
          return;
        case "new":
          setTip(null);
          setNote("New thread: the next message continues from nothing.");
          return;
        case "help":
          setNote(SLASH_HELP);
          return;
      }
    }
    setStream("");
    setReasoning("");
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
          session: tip ? turns.find((t) => t.id === tip)?.session : undefined,
        },
      ]);
      setTip(session_id);
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      // The local server answers the agent route with a 503 whose message
      // starts "agent not available" when no provider key is in the
      // environment (serve/agent.rs::start_turn); that is a configuration
      // fact, not a failure to report as a red error.
      if (/agent not (configured|available)/i.test(message))
        setUnavailable(true);
      else setError(message);
    }
  };

  return (
    <section
      className={`chat-dock${collapsed ? " collapsed" : ""}${onToggleCollapsed ? "" : " chat-dock--pane"}`}
      aria-label="Agent chat"
    >
      <header className="chat-head">
        {onToggleCollapsed ? (
          <button
            className="chat-toggle"
            aria-expanded={!collapsed}
            onClick={onToggleCollapsed}
            data-tip={collapsed ? "Show conversation" : "Hide conversation"}
          >
            <span aria-hidden="true">{collapsed ? "▴" : "▾"}</span> Agent
          </button>
        ) : (
          <span className="chat-toggle chat-toggle--static">Agent</span>
        )}
        {turns.length > 0 && (
          <span className="chat-count">
            {branch.length} of {turns.length} turn
            {turns.length === 1 ? "" : "s"} on this branch
          </span>
        )}
        {totals && hasUsage(totals) && (
          <span className="chat-stats muted" data-tip={statsTooltip(totals)}>
            {statsLine(totals)}
          </span>
        )}
        {tip && (
          <button
            className="btn-link chat-new"
            onClick={() => setTip(null)}
            data-tip="Start a conversation that does not continue from any existing turn"
          >
            New thread
          </button>
        )}
        {turns.length > 0 && (
          <button
            className={`btn-link chat-tree-toggle${showTree ? " chat-tree-toggle--on" : ""}`}
            onClick={() => setShowTree((v) => !v)}
            aria-pressed={showTree}
            data-tip="Zoom out: every turn as a node, branches where you rewound (/tree)"
          >
            Tree
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
            data-tip="Model id for the next turn — leave empty for the provider's default"
            onChange={(e) => setModel(e.target.value)}
          />
        </span>
      </header>

      {!collapsed && (
        <div className="chat-log" ref={logRef}>
          {branch.length === 0 && !running && (
            <p className="muted chat-empty">
              Ask the agent to edit, extend, or verify this document. It works
              through the document and its generated output, and every session
              is saved into the project as a <code>hick:session</code> file.
            </p>
          )}
          {showTree ? (
            <ChatTree
              turns={turns}
              tip={tip}
              onSelect={(id) => {
                setTip(id);
                setShowTree(false);
              }}
            />
          ) : (
            branch.map((turn) => {
              const kids = tip ? childrenOf(turns, turn.parent_id) : [];
              const siblings = kids.length > 1 ? kids : [];
              const at = siblings.findIndex((k) => k.id === turn.id);
              const isTip = turn.id === tip;
              const controls = (
                <div className="chat-turn-actions">
                  {siblings.length > 1 && (
                    <span
                      className="chat-branch"
                      data-tip="This turn has alternatives — the other branches from the same point"
                    >
                      <button
                        className="btn-link"
                        disabled={at <= 0}
                        onClick={() =>
                          setTip(deepestFrom(turns, siblings[at - 1].id))
                        }
                        aria-label="Previous branch"
                      >
                        ‹
                      </button>
                      {at + 1}/{siblings.length}
                      <button
                        className="btn-link"
                        disabled={at >= siblings.length - 1}
                        onClick={() =>
                          setTip(deepestFrom(turns, siblings[at + 1].id))
                        }
                        aria-label="Next branch"
                      >
                        ›
                      </button>
                    </span>
                  )}
                  {!isTip && turn.status !== "running" && (
                    <button
                      className="btn-link chat-rewind"
                      onClick={() => setTip(turn.id)}
                      data-tip="Continue from here — later turns stay on their own branch"
                    >
                      rewind here
                    </button>
                  )}
                  {turn.session && onOpenSession && (
                    <button
                      className="btn-link chat-open-session"
                      onClick={() => onOpenSession(turn.session!)}
                      data-tip={`Open the session file this turn is recorded in (${turn.session})`}
                    >
                      session
                    </button>
                  )}
                </div>
              );
              const view = asSessionTurn(turn);
              if (turn.id === running) {
                return (
                  <TurnCard
                    key={turn.id}
                    turn={view}
                    extra={controls}
                    live={{ text: stream, reasoning }}
                  />
                );
              }
              if (turn.status === "error") {
                return (
                  <article key={turn.id} className="chat-turn">
                    <div className="chat-msg chat-user">
                      <span className="chat-role">you</span>
                      <p>{turn.prompt}</p>
                      {controls}
                    </div>
                    <div className="chat-msg chat-agent">
                      <span className="chat-role">agent</span>
                      <p className="chat-error">
                        {turn.error ?? "session failed"}
                      </p>
                    </div>
                  </article>
                );
              }
              return (
                <TurnCard
                  key={turn.id}
                  turn={view}
                  extra={controls}
                  loadWork={loadWorkFor(turn)}
                />
              );
            })
          )}
        </div>
      )}

      <div className="chat-composer">
        <textarea
          value={prompt}
          rows={collapsed ? 1 : 2}
          placeholder={
            tip
              ? "Reply, or rewind to an earlier turn to branch…"
              : "Ask the agent…"
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
          The agent needs a provider API key. Set <code>ANTHROPIC_API_KEY</code>{" "}
          (or <code>OPENAI_API_KEY</code>, <code>DEEPSEEK_API_KEY</code>,{" "}
          <code>XAI_API_KEY</code>) in your environment and reopen this folder.
        </p>
      )}
      {note && <p className="chat-note muted">{note}</p>}
      {error && <p className="chat-note error">{error}</p>}
    </section>
  );
}

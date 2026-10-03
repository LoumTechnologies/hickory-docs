import type { AgentEditorContext } from "../api/agentTypes";
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
import { FeatureSettings } from "./FeatureSettings";
import { loadCollapsedLineage, saveCollapsedLineage } from "../lib/conversationLineage";
import { api } from "../api/client";
import { AcpControls, useAcp } from "./AcpControls";
import { AgentPicker, preferredAgent, rememberAgent } from "./AgentPicker";
import type {
  AgentTotals,
  AgentTurn,
  AgentWsEvent,
  RunWsMessage,
  SessionStep,
  SessionTurn,
  TranscriptEvent,
} from "../api/types";
import type { Realtime } from "../api/realtime";
import { TurnCard } from "./SessionTurns";
import { SessionLens } from "../views/SessionLens";
import { ChatTree } from "./ChatTree";
import { StopMark } from "./icons";

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
  getContext?: () => Promise<AgentEditorContext>;
  contextLabel?: string;
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
 *   /rerun       start a SECOND conversation, from what you learned in this one
 *   /tree        zoom out: the conversation as a tree (toggle)
 *   /new         start a thread that continues from nothing
 *   /help        list these
 *
 * Rewind and re-run are two different acts and the difference is only about
 * what is KEPT (docs/specs/freeform/sessions-you-run-again.md):
 *
 *   rewind — one session file, one conversation, an abandoned branch in the
 *            tree. The change of mind is kept, visible in the file forever.
 *   re-run — a second session file; the first is a draft you may discard, and
 *            nothing in session 2 refers to session 1. The change of mind is
 *            NOT kept.
 *
 * A re-run is not a reenactment: every byte of the second session is written
 * by the harness, the tools really run, and it stands without the first ever
 * having existed.
 *
 * Anything else starting with "/" is a message, not a command — a path or a
 * date at the start of a sentence must not be eaten.
 */
export type SlashCommand =
  | { kind: "rewind"; steps: number }
  | { kind: "rerun" }
  | { kind: "tree" }
  | { kind: "new" }
  | { kind: "help" };

export function parseSlash(text: string): SlashCommand | null {
  const m = text.trim().match(/^\/(rewind|rerun|tree|new|help)(?:\s+(\d+))?$/i);
  if (!m) return null;
  switch (m[1].toLowerCase()) {
    case "rewind":
      return { kind: "rewind", steps: m[2] ? Math.max(1, Number(m[2])) : 1 };
    case "rerun":
      return { kind: "rerun" };
    case "tree":
      return { kind: "tree" };
    case "new":
      return { kind: "new" };
    default:
      return { kind: "help" };
  }
}

/** What each act keeps — shown where the choice is made, not in a doc. */
export const REWIND_TIP =
  "Rewind — continue from here, in THIS conversation. The turns after it stay on their own branch in the same session file, so the change of mind is kept and visible forever.";

export const RERUN_TIP =
  "Re-run — start a SECOND conversation from what you learned in this one. The first becomes a draft you may discard (sessions/ is gitignored), and nothing in the second refers to it. The change of mind is not kept.";

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
  "/rewind [N] — back N turns, keeping the branch · /rerun — a second conversation, discarding this one · /tree — zoom out to the tree · /new — start a thread · /help";

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
  getContext,
  contextLabel,
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
  // A stop was asked for and its terminal frame has not arrived yet. The
  // button stays pressed-looking rather than clickable twice.
  const [stopping, setStopping] = useState(false);
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
  const [showCollapsedLineage, setShowCollapsedLineage] = useState(loadCollapsedLineage);
  const [backend, setBackend] = useState(preferredAgent);
  const [sending, setSending] = useState(false);
  const sendingRef = useRef(false);
  const earlyEvents = useRef<RunWsMessage[]>([]);
  const [provider, setProvider] = useState("");
  const [model, setModel] = useState("");

  const selectedTurn = turns.find(t => t.id === tip);
  const selectedBackend = selectedTurn?.provider.startsWith("acp:") ? selectedTurn.provider.slice(4) : "builtin";
  useEffect(() => { if (selectedTurn && !running) setBackend(selectedBackend); }, [tip, selectedBackend, running]);
  const selectedSession = selectedTurn?.session;
  const acp = useAcp(docId, selectedTurn ? selectedBackend : backend, selectedSession, running);
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
          setTotals(r.totals);
          // Hydrate the controls once — later polls must not clobber a
          // choice being made in the select/input mid-conversation.
          if (!hydratedRef.current) {
            hydratedRef.current = true;
            setTip(newestTurn(r.turns)?.id ?? null);
            setBackend(r.turns.length ? r.backend ?? "builtin" : preferredAgent());
            setRunning(r.turns.find(t => t.status === "running")?.id ?? null);
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
        if (!("run_id" in msg)) return;
        if (sendingRef.current && !runningRef.current) { earlyEvents.current.push(msg); return; }
        if (msg.run_id !== runningRef.current) return;
        if ("event" in msg) {
          const e = msg.event;
          setStream((prev) => appendStream(prev, e));
          setReasoning((prev) => appendReasoning(prev, e));
          return;
        }
        setRunning(null);
        setStopping(false);
        setStream("");
        setReasoning("");
        void refresh();
        onFinishedRef.current();
      }),
    [realtime, refresh],
  );

  // The hand on the cord. The run halts at its next seam — mid-stream
  // included, which is what stops the token spend on a model looping — and
  // the terminal "stopped" frame above clears the running state.
  const stop = useCallback(() => {
    if (!runningRef.current || stopping) return;
    setStopping(true);
    api.agentStop(docId).catch(() => {
      // The turn finished in the race between seeing it run and clicking:
      // the terminal frame is on its way and will clear everything.
      setStopping(false);
    });
  }, [docId, stopping]);

  const branch = useMemo(() => branchOf(turns, tip), [turns, tip]);
  // The conversation's record, when a turn has been written to one. The
  // lens draws every finished turn from the file; a turn still running, or
  // one that failed before it was recorded, is drawn as a card until then.
  const sessionPath = useMemo(
    () => [...branch].reverse().find((turn) => turn.session)?.session ?? null,
    [branch],
  );
  const inLens = useCallback(
    (turn: AgentTurn) => !!sessionPath && turn.session === sessionPath && turn.status === "ok",
    [sessionPath],
  );
  const lensStamp = turns.map((turn) => `${turn.id}:${turn.status}`).join(",");

  useEffect(() => {
    const log = logRef.current;
    if (log) log.scrollTop = log.scrollHeight;
  }, [branch.length, stream, collapsed]);

  useEffect(() => {
    if (!running) return;
    const timer = setInterval(() => {
      void api.agentTurns(docId).then(r => {
        const turn = r.turns.find(t => t.id === runningRef.current);
        if (turn && turn.status !== "running") {
          setRunning(null); setStopping(false); setStream(""); setReasoning("");
          void refresh(); onFinishedRef.current();
        }
      }, () => undefined);
    }, 1000);
    return () => clearInterval(timer);
  }, [running, docId, refresh]);

  const send = async () => {
    const text = prompt.trim();
    if (!text || running || sendingRef.current || (backend !== "builtin" && (!acp.state?.ready || acp.busy))) return;
    setError(null);
    setNote(null);
    const slash = backend !== "builtin" && !acp.state?.canRewind && text.startsWith("/rewind") ? null : parseSlash(text);
    if (backend !== "builtin" && !acp.state?.canRewind && text.startsWith("/rewind")) { setNote("This agent continues from its latest turn. Use New thread to start another conversation."); return; }
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
        case "rerun": {
          // A re-run is not a branch of this conversation: it is a SECOND
          // conversation, in its own session file, that stands without this
          // one. So the tip goes to nothing — the next message opens the new
          // session — and what carries across is whatever you distil by hand.
          setTip(null);
          setNote(
            "Re-run: the next message starts a SECOND conversation, in its own " +
              "session file. This one becomes a draft you may discard — sessions/ " +
              "is gitignored, so that is already the default — and nothing in the " +
              "new one will refer to it. Distil what you learned first: " +
              "`hick ingest --from carry <session>` writes the opening prompt and leaves the " +
              "tests you kept and the approaches you ruled out for you to fill in. " +
              "To keep the change of mind instead, rewind.",
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
    setSending(true); sendingRef.current = true; earlyEvents.current = [];
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
        backend,
        ...(getContext ? [await getContext()] as const : []),
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
          provider: backend === "builtin" ? provider || "anthropic" : `acp:${backend}`,
          model: backend === "builtin" ? model.trim() || defaultModelFor(provider) || "claude-sonnet-5" : "",
          usage: null,
          session: tip ? turns.find((t) => t.id === tip)?.session : undefined,
        },
      ]);
      setTip(session_id);
      const buffered = earlyEvents.current.filter(m => "run_id" in m && m.run_id === session_id);
      for (const msg of buffered) {
        if ("event" in msg) { setStream(prev => appendStream(prev, msg.event)); setReasoning(prev => appendReasoning(prev, msg.event)); }
        else if ("status" in msg) { setRunning(null); void refresh(); onFinishedRef.current(); }
      }
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      // The local server answers the agent route with a 503 whose message
      // starts "agent not available" when no provider key is in the
      // environment (serve/agent.rs::start_turn); that is a configuration
      // fact, not a failure to report as a red error.
      if (/agent not (configured|available)/i.test(message))
        setUnavailable(true);
      else setError(message);
    } finally { sendingRef.current = false; setSending(false); earlyEvents.current = []; }
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
            disabled={running !== null || sending}
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
        {!collapsed && <FeatureSettings label="Agent settings">
        {backend === "builtin" && <span className="chat-model" role="group" aria-label="Model choice">
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
        </span>}
      <AgentPicker backend={backend} agents={acp.agents} disabled={running !== null || sending} detecting={acp.detecting} error={acp.catalogueError}
        onRefresh={() => void acp.refreshAgents()} onChange={next => {
          rememberAgent(next); setBackend(next); setTip(null); setUnavailable(false); setError(null); setNote(null);
        }} />
        <AcpControls doc={docId} backend={backend} running={running !== null} control={acp} settingsOnly />
        <label><input type="checkbox" checked={showCollapsedLineage} onChange={event => {
          setShowCollapsedLineage(event.target.checked); saveCollapsedLineage(event.target.checked);
        }} />Show lineage for collapsed conversation items</label>
        </FeatureSettings>}
      </header>
      {!collapsed && <AcpControls doc={docId} backend={backend} running={running !== null} control={acp} settingsOpen={false} />}

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
            <>
              {/* The recorded turns, as the session document itself: line
                  numbers, the same cards, and ribbons from what each turn
                  read, wrote and pointed at. A lens, read-only. */}
              {sessionPath && <SessionLens path={sessionPath} stamp={lensStamp} showCollapsedLineage={showCollapsedLineage} />}
              {sessionPath && branch.some((turn) => inLens(turn)) && (
                <div className="chat-turn-strip" aria-label="Turns">
                  {branch.filter(inLens).map((turn) => (
                    <span key={turn.id} className="chat-turn-strip__turn">
                      <span className="muted">{turn.prompt.slice(0, 40)}</span>
                      {(backend === "builtin" || acp.state?.canRewind) && turn.id !== tip && (
                        <button
                          className="btn-link chat-rewind"
                          onClick={() => setTip(turn.id)}
                          data-tip={REWIND_TIP}
                        >
                          rewind here
                        </button>
                      )}
                    </span>
                  ))}
                  {onOpenSession && (
                    <button
                      className="btn-link chat-open-session"
                      onClick={() => onOpenSession(sessionPath)}
                      data-tip={`Open the session file this conversation is recorded in (${sessionPath})`}
                    >
                      session
                    </button>
                  )}
                </div>
              )}
              {branch.filter((turn) => !inLens(turn)).map((turn) => {
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
                  {(backend === "builtin" || acp.state?.canRewind) && !isTip && turn.status !== "running" && (
                    <button
                      className="btn-link chat-rewind"
                      onClick={() => setTip(turn.id)}
                      data-tip={REWIND_TIP}
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
              if (turn.status === "error" || turn.status === "stopped") {
                // A stop is the user's own act — quiet words, never the red
                // an actual failure gets.
                const stopped = turn.status === "stopped";
                return (
                  <article key={turn.id} className="chat-turn">
                    <div className="chat-msg chat-user">
                      <span className="chat-role">you</span>
                      <div className="chat-bubble">
                        <p>{turn.prompt}</p>
                      </div>
                      {controls}
                    </div>
                    <div className="chat-msg chat-agent">
                      <span className="chat-role">agent</span>
                      <div className="chat-bubble">
                        {stopped ? (
                          <p className="chat-stopped muted">
                            Stopped by you. Whatever it had already done is
                            real and recorded in the session; the next message
                            {turn.provider.startsWith("acp:") ? "continues with the agent’s retained context." : "continues as if this turn never ran."}
                          </p>
                        ) : (
                          <p className="chat-error">
                            {turn.error ?? "session failed"}
                          </p>
                        )}
                      </div>
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
              })}
            </>
          )}
        </div>
      )}

      {contextLabel && <p className="chat-note muted">Context: {contextLabel}. Current editor text is included when you send.</p>}
      <div className="chat-composer">
        {/* The draft is a bubble too, tail on your side: what you are
            typing is the next thing you will have said. */}
        <div className="chat-bubble chat-bubble--draft">
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
        </div>
        {running ? (
          // The way out of a runaway turn — a model looping mid-stream bills
          // tokens until somebody pulls this. Never disabled while a run is
          // live; "Stopping…" only means the request is in flight.
          <button
            className="btn chat-stop"
            disabled={stopping}
            onClick={stop}
            aria-label="Stop the agent"
            data-tip="Stop this turn now — the stream is cut and no more tokens are spent"
          >
            <StopMark /> {stopping ? "Stopping…" : "Stop"}
          </button>
        ) : (
          <button
            className="btn btn-primary"
            disabled={!prompt.trim() || sending || (backend !== "builtin" && (!acp.state?.ready || acp.busy))}
            onClick={() => void send()}
          >
            {sending ? "Starting…" : "Send"}
          </button>
        )}
      </div>
      {unavailable && (
        <p className="chat-note muted">
          The agent needs a provider API key. Add one in <a href="#/settings">Settings</a>, then try again.
        </p>
      )}
      {note && <p className="chat-note muted">{note}</p>}
      {error && <p className="chat-note error">{error}</p>}
    </section>
  );
}

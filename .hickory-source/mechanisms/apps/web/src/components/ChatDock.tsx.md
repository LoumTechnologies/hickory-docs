<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/ChatDock.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-57557cac78c8"></a>
## `apps/web/src/components/ChatDock.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Missing Key Degrades to a Note, Never a Crash](../../../../../guarantees/agent/a-missing-key-degrades-to-a-note/contract.md), [A running agent can be stopped](../../../../../guarantees/agent/a-running-agent-can-be-stopped/contract.md), [Read the retained branch of an agent conversation](../../../../../guarantees/agent/a-session-is-the-conversation/contract.md), [ACP Agents Work In The Agent Pane And Leave A Hickory Record](../../../../../guarantees/agent/acp-agents-are-first-class/contract.md), [Trace conversation passages without confusing context with support](../../../../../guarantees/agent/an-answer-in-the-agent-pane-has-ribbons/contract.md), [Keep model reasoning distinct from the answer](../../../../../guarantees/agent/reasoning-is-shown-apart-from-the-answer/contract.md), [Rewind And Re-Run Are Named Apart, And The Difference Is What Is Kept](../../../../../guarantees/agent/rewind-and-re-run-are-two-different-acts/contract.md), [The Dock Reports Spend and Runs the Chosen Model](../../../../../guarantees/agent/the-dock-reports-spend-and-runs-the-chosen-model/contract.md), [Unsent responses survive restart](../../../../../guarantees/agent/unsent-responses-survive-restart/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-57557cac78c8-PROVIDERS"></a>
### `PROVIDERS`

<storycode:copy id="f-57557cac78c8-PROVIDERS">
import { useConversationActions } from "./uiConversationActions";
import { ConversationModelChoice } from "./ConversationModelChoice";
import { UiCheckbox } from "./UiCheckbox";
import { ContextUpdates } from "./ContextUpdates";
import { registerInvestigationConversation } from "../lib/reviewInvestigations";
import { AgentProject } from "./AgentProject";
import { AgentChanges } from "./AgentChanges";
import { acpApi } from "../api/acp";
import type { AgentChange } from "../api/acp";
import type { AgentEditorContext } from "../api/agentTypes";
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
  TranscriptEvent,
} from "../api/types";
import type { Realtime } from "../api/realtime";
import { ConversationEditor, focusConversationResponse } from "../views/ConversationEditor";
import { ChatTree } from "./ChatTree";
import { StopMark } from "./icons";
import { useResponseRecovery } from "../lib/responseRecovery";

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

</storycode:copy>
<a id="f-57557cac78c8-defaultModelFor"></a>
### `defaultModelFor`

The model a provider runs when none is typed, for the placeholder.

<storycode:copy id="f-57557cac78c8-defaultModelFor">
/** The model a provider runs when none is typed, for the placeholder. */
export function defaultModelFor(provider: string): string {
  return PROVIDERS.find((p) => p.id === provider)?.defaultModel ?? "";
}

</storycode:copy>
<a id="f-57557cac78c8-formatTokens"></a>
### `formatTokens`

<storycode:copy id="f-57557cac78c8-formatTokens">
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

</storycode:copy>
<a id="f-57557cac78c8-formatUsd"></a>
### `formatUsd`

Session cost to four decimal places; "$—" when a model's price is
unknown (the server reports unknown, never a guess).

<storycode:copy id="f-57557cac78c8-formatUsd">
/** Session cost to four decimal places; "$—" when a model's price is
 * unknown (the server reports unknown, never a guess). */
export function formatUsd(usd: number | null): string {
  return usd === null ? "$—" : `$${usd.toFixed(4)}`;
}

</storycode:copy>
<a id="f-57557cac78c8-cacheHitRate"></a>
### `cacheHitRate`

Cache hit rate = reads / (uncached input + reads); null before any
input-bearing turn, so the dock can show "—" instead of a fake 0%.

<storycode:copy id="f-57557cac78c8-cacheHitRate">
/** Cache hit rate = reads / (uncached input + reads); null before any
 * input-bearing turn, so the dock can show "—" instead of a fake 0%. */
export function cacheHitRate(input: number, cacheRead: number): number | null {
  const denominator = input + cacheRead;
  return denominator === 0 ? null : cacheRead / denominator;
}

</storycode:copy>
<a id="f-57557cac78c8-statsLine"></a>
### `statsLine`

The dock header's one-line spend summary,
e.g. "$0.0342 · in 12.4k · out 3.1k · cache 78%".

<storycode:copy id="f-57557cac78c8-statsLine">
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

</storycode:copy>
<a id="f-57557cac78c8-statsTooltip"></a>
### `statsTooltip`

The tooltip behind the stats line: all four counters, uncompacted.

<storycode:copy id="f-57557cac78c8-statsTooltip">
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

</storycode:copy>
<a id="f-57557cac78c8-hasUsage"></a>
### `hasUsage`

Whether any turn has reported usage yet — before that the stats line
would be all zeros, which reads as free rather than unstarted.

<storycode:copy id="f-57557cac78c8-hasUsage">
/** Whether any turn has reported usage yet — before that the stats line
 * would be all zeros, which reads as free rather than unstarted. */
export function hasUsage(t: AgentTotals): boolean {
  return t.input + t.output + t.cache_read + t.cache_write > 0;
}

</storycode:copy>
<a id="f-57557cac78c8-ChatDockProps"></a>
### `ChatDockProps`

<storycode:copy id="f-57557cac78c8-ChatDockProps">
export interface ChatDockProps {
  initialPrompt?: string;
  docId: string;
  /** Folder identity invalidates queued recovery when this window attaches a folder. */
  recoveryWorkspace?: string;
  getContext?: () => Promise<AgentEditorContext>;
  applyAgentChange?: (change: AgentChange) => void | Promise<void>;
  contextLabel?: string;
  contextControls?: import("react").ReactNode;
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

</storycode:copy>
<a id="f-57557cac78c8-appendStream"></a>
### `appendStream`

Fold one live agent event into the streaming preview.
Tokens append as they arrive; a script or tool start becomes a one-line
marker so the pause while it runs reads as work, not a stall. Everything
else (usage, thinking, completions) is bookkeeping the finished turn will
render properly, so it adds nothing here.

<storycode:copy id="f-57557cac78c8-appendStream">
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

</storycode:copy>
<a id="f-57557cac78c8-appendReasoning"></a>
### `appendReasoning`

The model's reasoning, streamed apart from its answer and shown folded.

<storycode:copy id="f-57557cac78c8-appendReasoning">
/** The model's reasoning, streamed apart from its answer and shown folded. */
export function appendReasoning(
  prev: string,
  e: TranscriptEvent | AgentWsEvent,
): string {
  return e.kind === "reasoning" ? prev + e.data : prev;
}

</storycode:copy>
<a id="f-57557cac78c8-SlashCommand"></a>
### `SlashCommand`

The composer's slash commands — the keyboard way to move around the tree.
/rewind      make the tip's parent the tip (one step back)
/rewind N    go back N turns
/rerun       start a SECOND conversation, from what you learned in this one
/tree        zoom out: the conversation as a tree (toggle)
/new         start a thread that continues from nothing
/help        list these
Rewind and re-run are two different acts and the difference is only about
what is KEPT (docs/specs/freeform/sessions-you-run-again.md):
rewind — one session file, one conversation, an abandoned branch in the
tree. The change of mind is kept, visible in the file forever.
re-run — a second session file; the first is a draft you may discard, and
nothing in session 2 refers to session 1. The change of mind is
NOT kept.
A re-run is not a reenactment: every byte of the second session is written
by the harness, the tools really run, and it stands without the first ever
having existed.
Anything else starting with "/" is a message, not a command — a path or a
date at the start of a sentence must not be eaten.

<storycode:copy id="f-57557cac78c8-SlashCommand">
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

</storycode:copy>
<a id="f-57557cac78c8-parseSlash"></a>
### `parseSlash`

<storycode:copy id="f-57557cac78c8-parseSlash">
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

</storycode:copy>
<a id="f-57557cac78c8-REWIND_TIP"></a>
### `REWIND_TIP`

What each act keeps — shown where the choice is made, not in a doc.

<storycode:copy id="f-57557cac78c8-REWIND_TIP">
/** What each act keeps — shown where the choice is made, not in a doc. */
export const REWIND_TIP =
  "Rewind — continue from here, in THIS conversation. The turns after it stay on their own branch in the same session file, so the change of mind is kept and visible forever.";

</storycode:copy>
<a id="f-57557cac78c8-RERUN_TIP"></a>
### `RERUN_TIP`

<storycode:copy id="f-57557cac78c8-RERUN_TIP">
export const RERUN_TIP =
  "Re-run — start a SECOND conversation from what you learned in this one. The first becomes a draft you may discard (sessions/ is gitignored), and nothing in the second refers to it. The change of mind is not kept.";

</storycode:copy>
<a id="f-57557cac78c8-rewindFrom"></a>
### `rewindFrom`

The turn `steps` back from `tip` along parent pointers (null = root).

<storycode:copy id="f-57557cac78c8-rewindFrom">
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

</storycode:copy>
<a id="f-57557cac78c8-SLASH_HELP"></a>
### `SLASH_HELP`

<storycode:copy id="f-57557cac78c8-SLASH_HELP">
export const SLASH_HELP =
  "/rewind [N] — back N turns, keeping the branch · /rerun — a second conversation, discarding this one · /tree — zoom out to the tree · /new — start a thread · /help";

</storycode:copy>
<a id="f-57557cac78c8-branchOf"></a>
### `branchOf`

Path from a root turn down to `tip`, oldest first.

<storycode:copy id="f-57557cac78c8-branchOf">
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

</storycode:copy>
<a id="f-57557cac78c8-childrenOf"></a>
### `childrenOf`

Children of `parent` (null = conversation roots), oldest first.

<storycode:copy id="f-57557cac78c8-childrenOf">
/** Children of `parent` (null = conversation roots), oldest first. */
export function childrenOf(
  turns: AgentTurn[],
  parent: string | null,
): AgentTurn[] {
  return turns.filter((t) => t.parent_id === parent);
}

</storycode:copy>
<a id="f-57557cac78c8-deepestFrom"></a>
### `deepestFrom`

Follow the newest child at each step — the tip a branch switch lands on.

<storycode:copy id="f-57557cac78c8-deepestFrom">
/** Follow the newest child at each step — the tip a branch switch lands on. */
export function deepestFrom(turns: AgentTurn[], from: string): string {
  let cursor = from;
  for (;;) {
    const kids = childrenOf(turns, cursor);
    if (kids.length === 0) return cursor;
    cursor = kids[kids.length - 1].id;
  }
}

</storycode:copy>
<a id="f-57557cac78c8-ChatDock"></a>
### `ChatDock`

The header identifies the selected provider/model or ACP adapter without adding a
configuration row. Empty conversations start with the response editor itself.
Context comparisons live in the gutter only when a retained reference differs;
editor identity remains available to the agent without an always-visible banner.

<storycode:copy id="f-57557cac78c8-ChatDock">
export function ChatDock({
  initialPrompt = "",
  docId,
  recoveryWorkspace = "window",
  getContext,
  applyAgentChange,
  contextLabel,
  contextControls,
  realtime,
  collapsed = false,
  onToggleCollapsed,
  onAgentFinished,
  onOpenSession,
}: ChatDockProps) {
  const [hydrated, setHydrated] = useState(false);
  const [turns, setTurns] = useState<AgentTurn[]>([]);
  const recovery = useResponseRecovery(docId, recoveryWorkspace, initialPrompt, turns, hydrated);
  const tip = recovery.state.tip;
  const setTip = recovery.select;
  const prompt = recovery.text;
  const setPrompt = recovery.edit;
  const [running, setRunning] = useState<string | null>(null);
  // A stop was asked for and its terminal frame has not arrived yet. The
  // button stays pressed-looking rather than clickable twice.
  const [stopping, setStopping] = useState(false);
  const [stream, setStream] = useState("");
  const [reasoning, setReasoning] = useState("");
  const [showTree, setShowTree] = useState(false);
  const [note, setNote] = useState<string | null>(null);
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
  const selectedBackend = selectedTurn?.backend ?? (selectedTurn?.provider.startsWith("acp:") ? selectedTurn.provider.slice(4) : "python");
  useEffect(() => { if (selectedTurn && !running) setBackend(selectedBackend); }, [tip, selectedBackend, running]);
  const selectedSession = selectedTurn?.session;
  const acp = useAcp(docId, selectedTurn ? selectedBackend : backend, selectedSession, running, hydrated);
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
          setHydrated(true);
          setTotals(r.totals);
          // Hydrate the controls once — later polls must not clobber a
          // choice being made in the select/input mid-conversation.
          if (!hydratedRef.current) {
            hydratedRef.current = true;
            setBackend(r.turns.length ? r.backend ?? "python" : preferredAgent());
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
  const stop = useCallback(async () => {
    if (!runningRef.current || stopping) return false;
    setStopping(true);
    try { await api.agentStop(docId); return true; }
    catch (reason) { setStopping(false); setError(String(reason)); return false; }
  }, [docId, stopping]);

  const branch = useMemo(() => branchOf(turns, tip), [turns, tip]);
  // The conversation's record, when a turn has been written to one. The
  // lens draws every finished turn from the file; a turn still running, or
  // one that failed before it was recorded, is drawn as a card until then.
  const sessionPath = useMemo(
    () => [...branch].reverse().find((turn) => turn.session)?.session ?? null,
    [branch],
  );
  const lensStamp = turns.map((turn) => `${turn.id}:${turn.status}`).join(",");

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

  const captureRequest = acp.state?.edits?.captureRequest;
  useEffect(() => {
    if (!running || !captureRequest || !getContext) return;
    let live = true;
    void getContext().then(async context => { if (live) await acpApi.edits(docId, { capture_request: captureRequest, context }, backend); })
      .catch(e => setError(String(e)));
    return () => { live = false; };
  }, [running, captureRequest, docId, backend]);

  const send = async (investigation?: string) => {
    const submitted = recovery.snapshot();
    const text = (investigation ?? prompt).trim();
    if (!text || running || sendingRef.current || (!acp.state?.ready || acp.busy)) {
      if (investigation) throw new Error("The selected conversation is busy or unavailable.");
      return;
    }
    setError(null);
    setNote(null);
    const slash = backend !== "python" && !acp.state?.canRewind && text.startsWith("/rewind") ? null : parseSlash(text);
    if (backend !== "python" && !acp.state?.canRewind && text.startsWith("/rewind")) { setNote("This agent continues from its latest turn. Use New thread to start another conversation."); return; }
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
      recovery.accepted(investigation ? { ...submitted, revision: -1 } : submitted, session_id);
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
          backend,
          provider: (backend === "python") ? provider || "anthropic" : `acp:${backend}`,
          model: (backend === "python") ? model.trim() || defaultModelFor(provider) || "claude-sonnet-5" : "",
          usage: null,
          session: tip ? turns.find((t) => t.id === tip)?.session : undefined,
        },
      ]);
      const buffered = earlyEvents.current.filter(m => "run_id" in m && m.run_id === session_id);
      for (const msg of buffered) {
        if ("event" in msg) { setStream(prev => appendStream(prev, msg.event)); setReasoning(prev => appendReasoning(prev, msg.event)); }
        else if ("status" in msg) { setRunning(null); void refresh(); onFinishedRef.current(); }
      }
      return session_id;
    } catch (e) {
      if (investigation) throw e;
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

  const conversationActions = useConversationActions({ doc: docId, draft: prompt, tip, turns, running, sending, stopping,
    ready: !!acp.state?.ready && !acp.busy, tree: showTree, collapsed,
    evidence: { stream, reasoning, totals, note, error, unavailable, recovery_error: recovery.error, backend, provider, model, context: contextLabel ?? null, session_path: sessionPath },
    setDraft: setPrompt, discard: recovery.discard, setTip, setTree: setShowTree,
    send: () => send(), stop, toggleCollapsed: onToggleCollapsed,
  });
  const investigationSend = useRef(send);
  investigationSend.current = send;
  useEffect(() => registerInvestigationConversation({
    id: docId,
    label: `${contextLabel ?? docId} · ${selectedSession ?? "new conversation"} · branch ${tip ?? "root"}`,
    send: async (text) => {
      const turn = await investigationSend.current(text);
      if (!turn) throw new Error("Investigation was not started.");
      return turn;
    },
  }), [docId, contextLabel, selectedSession, tip]);

  return (
    <section
      className={`chat-dock${collapsed ? " collapsed" : ""}${onToggleCollapsed ? "" : " chat-dock--pane"}`}
      aria-label="Agent chat"
      data-coach-target="agent"
    >
      {!collapsed && backend === "python" && <ContextUpdates key={running ?? tip ?? "root"} doc={docId} branch={running ?? tip} getContext={getContext} />}
      <header className="chat-head">
        {onToggleCollapsed ? (
          <button
            className="chat-toggle"
            aria-expanded={!collapsed}
            onClick={() => void conversationActions.invoke("collapse")}
            data-tip={collapsed ? "Show conversation" : "Hide conversation"}
          >
            <span aria-hidden="true">{collapsed ? "▴" : "▾"}</span> Agent
          </button>
        ) : (
          <span className="chat-toggle chat-toggle--static">Agent</span>
        )}
        <span className="chat-identity" title={backend === "python" ? "Python agent · provider and selected model" : "ACP adapter · reported model"}>
          {backend === "python" ? `${PROVIDERS.find(p => p.id === provider)?.label ?? provider} · ${model.trim() || defaultModelFor(provider)}` : `${acp.agents.find(a => a.id === backend)?.name ?? backend} · ACP · ${acp.state?.configOptions?.find(o => o.category === "model" || o.id === "model")?.currentValue || acp.state?.models?.currentModelId || "Model not reported"}`}
        </span>
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
            onClick={() => void conversationActions.invoke("new")}
            data-tip="Start a conversation that does not continue from any existing turn"
          >
            New thread
          </button>
        )}
        {turns.length > 0 && (
          <button
            className={`btn-link chat-tree-toggle${showTree ? " chat-tree-toggle--on" : ""}`}
            onClick={() => void conversationActions.invoke("tree", { open: !showTree })}
            aria-pressed={showTree}
            data-tip="Zoom out: every turn as a node, branches where you rewound (/tree)"
          >
            Tree
          </button>
        )}
        {!collapsed && <FeatureSettings label="Agent settings" owner={docId}>
        {backend === "python" && <ConversationModelChoice scope={`${docId}:${tip ?? "root"}:${selectedSession ?? "new"}`} provider={provider} model={model} disabled={running !== null || sending} providers={PROVIDERS} defaultModel={defaultModelFor(provider)} onProvider={setProvider} onModel={setModel} />}
      <AgentPicker scope={`${docId}:${tip ?? "root"}:${selectedSession ?? "new"}`} backend={backend} agents={acp.agents} disabled={running !== null || sending} detecting={acp.detecting} error={acp.catalogueError}
        onRefresh={() => acp.refreshAgents()} onChange={next => {
          rememberAgent(next); setBackend(next); setTip(null); setUnavailable(false); setError(null); setNote(null);
        }} />
        <AcpControls doc={docId} backend={backend} running={running !== null} control={acp} settingsOnly />
        {contextControls}
        <label><UiCheckbox label="conversation collapsed lineage" checked={showCollapsedLineage} onCheckedChange={checked => {
          setShowCollapsedLineage(checked); saveCollapsedLineage(checked);
        }} />Show lineage for collapsed conversation items</label>
        </FeatureSettings>}
      </header>
      {!collapsed && <AcpControls doc={docId} backend={backend} running={running !== null} control={acp} settingsOpen={false} showTools={false} />}

      {!collapsed && (
        <div className="chat-log" ref={logRef}>
          <AgentChanges doc={docId} state={acp.state} running={running !== null} apply={applyAgentChange} update={acp.setState} />
          {showTree ? (
            <ChatTree
              turns={turns}
              tip={tip}
              onSelect={id => { if (!running && !sending) void conversationActions.invoke("tip", { id }); }}
            />
          ) : (
            <>
              {branch.filter(turn => turn.status === "ok").map(turn => <AgentProject key={turn.id} doc={docId} turn={turn.id} getContext={getContext} applyChange={applyAgentChange} />)}
              <ConversationEditor branch={branch} path={sessionPath} stamp={lensStamp} running={running}
                showCollapsedLineage={showCollapsedLineage} stream={stream} reasoning={reasoning} draft={prompt} onDraft={setPrompt}
                onSend={() => void conversationActions.invoke("send")} sending={sending} canSend={!running && !sending && (!!acp.state?.ready && !acp.busy)} />
              {branch.length > 0 && <div className="chat-turn-strip" aria-label="Turns">
                {branch.map(turn => <span key={turn.id} className="chat-turn-strip__turn">
                  <span className="muted">{turn.prompt.slice(0, 40)}</span>
                  {(backend === "python" || acp.state?.canRewind) && turn.id !== tip &&
                    <button className="btn-link chat-rewind" disabled={!!running || sending} onClick={() => void conversationActions.invoke("tip", { id: turn.id })} data-tip={REWIND_TIP}>rewind here</button>}
                </span>)}
                {sessionPath && onOpenSession && <button className="btn-link" onClick={() => onOpenSession(sessionPath)}>session</button>}
              </div>}
            </>
          )}
        </div>
      )}

      <div className="chat-composer">
        <span className="muted">{!collapsed && !showTree ? <button className="btn-link" onClick={() => focusConversationResponse(logRef.current)}>Write a response</button> : "Type your response at the bottom"} · ⌘/Ctrl+Enter to send</span>
        {running ? (
          // The way out of a runaway turn — a model looping mid-stream bills
          // tokens until somebody pulls this. Never disabled while a run is
          // live; "Stopping…" only means the request is in flight.
          <button
            className="btn chat-stop"
            disabled={stopping}
            onClick={() => void conversationActions.invoke("stop")}
            aria-label="Stop the agent"
            data-tip="Stop this turn now — the stream is cut and no more tokens are spent"
          >
            <StopMark /> {stopping ? "Stopping…" : "Stop"}
          </button>
        ) : (collapsed || showTree) ? (
          <button
            className="btn btn-primary"
            disabled={!prompt.trim() || sending || (!acp.state?.ready || acp.busy)}
            onClick={() => void conversationActions.invoke("send")}
          >
            {sending ? "Starting…" : "Send"}
          </button>
        ) : null}
      </div>
      {prompt && <button className="btn-link" onClick={() => void conversationActions.invoke("discard")}>Discard response</button>}
      {recovery.error && <p className="chat-note error" role="status">{recovery.error}</p>}
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
</storycode:copy>

### Assemble `apps/web/src/components/ChatDock.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ChatDock.tsx"><storycode:paste select="#f-57557cac78c8-PROVIDERS"/><storycode:paste select="#f-57557cac78c8-defaultModelFor"/><storycode:paste select="#f-57557cac78c8-formatTokens"/><storycode:paste select="#f-57557cac78c8-formatUsd"/><storycode:paste select="#f-57557cac78c8-cacheHitRate"/><storycode:paste select="#f-57557cac78c8-statsLine"/><storycode:paste select="#f-57557cac78c8-statsTooltip"/><storycode:paste select="#f-57557cac78c8-hasUsage"/><storycode:paste select="#f-57557cac78c8-ChatDockProps"/><storycode:paste select="#f-57557cac78c8-appendStream"/><storycode:paste select="#f-57557cac78c8-appendReasoning"/><storycode:paste select="#f-57557cac78c8-SlashCommand"/><storycode:paste select="#f-57557cac78c8-parseSlash"/><storycode:paste select="#f-57557cac78c8-REWIND_TIP"/><storycode:paste select="#f-57557cac78c8-RERUN_TIP"/><storycode:paste select="#f-57557cac78c8-rewindFrom"/><storycode:paste select="#f-57557cac78c8-SLASH_HELP"/><storycode:paste select="#f-57557cac78c8-branchOf"/><storycode:paste select="#f-57557cac78c8-childrenOf"/><storycode:paste select="#f-57557cac78c8-deepestFrom"/><storycode:paste select="#f-57557cac78c8-ChatDock"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-e7bd6d898a44"></a>
## `apps/web/src/components/ChatDock.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A running agent can be stopped](../../../../../guarantees/agent/a-running-agent-can-be-stopped/contract.md), [Read the retained branch of an agent conversation](../../../../../guarantees/agent/a-session-is-the-conversation/contract.md), [ACP Agents Work In The Agent Pane And Leave A Hickory Record](../../../../../guarantees/agent/acp-agents-are-first-class/contract.md), [Separate model snippets from trusted host operations](../../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md), [Rewind And Re-Run Are Named Apart, And The Difference Is What Is Kept](../../../../../guarantees/agent/rewind-and-re-run-are-two-different-acts/contract.md), [The Agent pane is a live document](../../../../../guarantees/agent/the-agent-pane-is-a-live-document/contract.md), [The Dock Reports Spend and Runs the Chosen Model](../../../../../guarantees/agent/the-dock-reports-spend-and-runs-the-chosen-model/contract.md), [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Unsent responses survive restart](../../../../../guarantees/agent/unsent-responses-survive-restart/contract.md).

<a id="f-e7bd6d898a44-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-e7bd6d898a44-expression_statement">
import { EditorView } from "@codemirror/view";
import { responseStart } from "../editor/protectedPrefix";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  cleanup,
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";

vi.mock("../api/client", () => ({ request: vi.fn().mockImplementation(async (_method, path: string) => path.endsWith("/agent/python/context") ? { items: [], updates: [], coverage: "No retained references in this fixture" } : {}),
  api: { responseDraft: vi.fn().mockResolvedValue({ scope: "test", state: null }), saveResponseDraft: vi.fn().mockResolvedValue({ ok: true }), agent: vi.fn(), agentTurns: vi.fn(), agentStop: vi.fn(), executor: vi.fn().mockResolvedValue({}), sessionView: vi.fn().mockRejectedValue(new Error("No session fixture")), complete: vi.fn().mockResolvedValue({ suggestions: [] }) },
}));

</storycode:copy>
<a id="f-e7bd6d898a44-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-e7bd6d898a44-expression_statement-2">
vi.mock("../api/acp", () => ({ acpApi: { catalogue: vi.fn().mockResolvedValue({ agents: [] }), connect: vi.fn(), state: vi.fn() } }));

</storycode:copy>
<a id="f-e7bd6d898a44-import_statement"></a>
### `import_statement`

<storycode:copy id="f-e7bd6d898a44-import_statement">
import {
  ChatDock,
  appendStream,
  branchOf,
  cacheHitRate,
  childrenOf,
  deepestFrom,
  formatTokens,
  formatUsd,
  statsLine,
  REWIND_TIP,
  RERUN_TIP,
  SLASH_HELP,
} from "./ChatDock";
</storycode:copy>
<a id="f-e7bd6d898a44-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-e7bd6d898a44-import_statement-2">
import { api } from "../api/client";
</storycode:copy>
<a id="f-e7bd6d898a44-import_statement-3"></a>
### `import_statement`

<storycode:copy id="f-e7bd6d898a44-import_statement-3">
import { acpApi } from "../api/acp";
</storycode:copy>
<a id="f-e7bd6d898a44-import_statement-4"></a>
### `import_statement`

<storycode:copy id="f-e7bd6d898a44-import_statement-4">
import type { AgentTotals, AgentTurn, AgentTurnsResponse } from "../api/types";
</storycode:copy>
<a id="f-e7bd6d898a44-import_statement-5"></a>
### `import_statement`

<storycode:copy id="f-e7bd6d898a44-import_statement-5">
import type { Realtime } from "../api/realtime";

</storycode:copy>
<a id="f-e7bd6d898a44-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-e7bd6d898a44-expression_statement-3">
afterEach(() => { localStorage.clear(); });

</storycode:copy>
<a id="f-e7bd6d898a44-expression_statement-4"></a>
### `expression_statement`

Protects docs/guarantees/agent/acp-agents-are-first-class.md.

<storycode:copy id="f-e7bd6d898a44-expression_statement-4">
// Protects docs/guarantees/agent/acp-agents-are-first-class.md.
describe("switching agents", () => {
  afterEach(() => { cleanup(); vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [] }); });
  it("detects adapters, remembers a choice, and sends an ACP turn from a new thread", async () => {
    const agents = [{ id: "python", name: "Hickory Python", command: "python", args: [], available: true }, { id: "codex", name: "Codex", command: "codex-acp", args: [], available: true },
      { id: "claude", name: "Claude Agent", command: "claude-agent-acp", args: [], available: false, installable: true }];
    vi.mocked(acpApi.catalogue).mockResolvedValue({ agents });
    vi.mocked(acpApi.connect).mockResolvedValue({ backend: "codex", ready: true, models: { currentModelId: "adapter-model" } });
    vi.mocked(acpApi.state).mockResolvedValue({ backend: "codex", ready: true, models: { currentModelId: "adapter-model" } });
    vi.mocked(api.agentTurns).mockResolvedValue({ turns: [turn("old", null)], backend: "python", provider: "anthropic", model: "claude-sonnet-5", totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
    vi.mocked(api.agent).mockResolvedValue({ session_id: "acp-turn" });
    const realtime = { onRunEvent: () => () => undefined } as unknown as Realtime;
    const props = { docId: "d1", realtime, onAgentFinished: () => undefined };
    const view = render(<ChatDock {...props} />);
    expect(screen.queryByRole("combobox", { name: "Agent" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Agent settings" }));
    await screen.findByRole("option", { name: "Codex (ACP)" });
    expect(screen.getByRole("option", { name: "Hickory Python" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "Claude Agent — install adapter" })).toBeTruthy();
    await waitFor(() => expect(document.querySelector(".cm-content")?.textContent).toContain("p:old"));
    fireEvent.change(screen.getByRole("combobox", { name: "Agent" }), { target: { value: "codex" } });
    await waitFor(() => expect(acpApi.connect).toHaveBeenCalledWith("d1", "codex", undefined));
    expect(localStorage.getItem("hickory.agent")).toBe("codex");
    expect(screen.getByText("Codex · ACP · adapter-model")).toBeTruthy();
    await typeResponse("Use Codex");
    await waitFor(() => expect((screen.getByRole("button", { name: "Send" }) as HTMLButtonElement).disabled).toBe(false));
    fireEvent.click(screen.getByRole("button", { name: "Send" }));
    await waitFor(() => expect(api.agent).toHaveBeenCalledWith("d1", "Use Codex", null, "anthropic", "", "codex"));
    await screen.findByRole("button", { name: "Stop the agent" });
    expect((screen.getByRole("combobox", { name: "Agent" }) as HTMLSelectElement).value).toBe("codex");
    view.unmount();
    vi.mocked(api.agentTurns).mockResolvedValue({ turns: [], backend: "python", provider: "anthropic", model: "claude-sonnet-5", totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
    render(<ChatDock {...props} />);
    fireEvent.click(screen.getByRole("button", { name: "Agent settings" }));
    await waitFor(() => expect((screen.getByRole("combobox", { name: "Agent" }) as HTMLSelectElement).value).toBe("codex"));
    vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [...agents, { id: "custom", name: "My agent", command: "my-acp", args: [], available: true }] });
    fireEvent.click(screen.getByRole("button", { name: "Refresh agents" }));
    await screen.findByRole("option", { name: "My agent (ACP)" });
  });
  // Guarantee: docs/guarantees/agent/python-execution-has-two-boundaries.md
  it("selects Python without ACP login, retains model choice and hydrates its backend", async () => {
    vi.mocked(acpApi.connect).mockClear();
    vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [{ id: "python", name: "Hickory Python", command: "/venv/bin/python", args: [], available: true, kind: "python" }] });
    vi.mocked(acpApi.state).mockResolvedValue({ backend: "python", ready: true, canRewind: true, edits: { mode: "review", changes: [] } });
    vi.mocked(api.agentTurns).mockResolvedValue({ turns: [], backend: "python", provider: "anthropic", model: "claude-sonnet-5", totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
    vi.mocked(api.agent).mockResolvedValue({ session_id: "python-turn" });
    const realtime = { onRunEvent: () => () => undefined } as unknown as Realtime;
    const props = { docId: "d1", realtime, onAgentFinished: () => undefined };
    const view = render(<ChatDock {...props} />);
    fireEvent.click(screen.getByRole("button", { name: "Agent settings" }));
    await screen.findByRole("option", { name: "Hickory Python" });
    fireEvent.change(screen.getByRole("combobox", { name: "Agent" }), { target: { value: "python" } });
    await waitFor(() => expect(acpApi.state).toHaveBeenCalledWith("d1", "python", ""));
    expect(acpApi.connect).not.toHaveBeenCalled();
    expect(screen.getByText("Anthropic · claude-sonnet-5")).toBeTruthy();
    expect(screen.queryByText(/Ask the Python agent/)).toBeNull();
    expect(screen.queryByText(/Context:.*Next context projection/)).toBeNull();
    expect(screen.queryByRole("region", { name: "Conversation document edits" })).toBeNull();
    expect(screen.queryByRole("button", { name: /Changed agent context/ })).toBeNull();
    expect(screen.getByRole("combobox", { name: "Provider" })).toBeTruthy();
    fireEvent.change(screen.getByRole("textbox", { name: "Model" }), { target: { value: "my-model" } });
    await typeResponse("Propose a note in Python");
    await waitFor(() => expect((screen.getByRole("button", { name: "Send" }) as HTMLButtonElement).disabled).toBe(false));
    fireEvent.click(screen.getByRole("button", { name: "Send" }));
    await waitFor(() => expect(api.agent).toHaveBeenCalledWith("d1", "Propose a note in Python", null, "anthropic", "my-model", "python"));
    await screen.findByRole("button", { name: "Stop the agent" });
    expect((screen.getByRole("combobox", { name: "Agent" }) as HTMLSelectElement).value).toBe("python");
    view.unmount();
    vi.mocked(api.agentTurns).mockResolvedValue({ turns: [{ ...turn("python-turn", null), backend: "python", session: "sessions/python.md" }], backend: "python", provider: "anthropic", model: "my-model", totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
    render(<ChatDock {...props} />);
    fireEvent.click(screen.getByRole("button", { name: "Agent settings" }));
    await waitFor(() => expect((screen.getByRole("combobox", { name: "Agent" }) as HTMLSelectElement).value).toBe("python"));
    expect(acpApi.connect).not.toHaveBeenCalled();
  });

});

</storycode:copy>
<a id="f-e7bd6d898a44-turn"></a>
### `turn`

<storycode:copy id="f-e7bd6d898a44-turn">
function turn(id: string, parent: string | null): AgentTurn {
  return {
    id,
    parent_id: parent,
    prompt: `p:${id}`,
    answer: `a:${id}`,
    status: "ok",
    error: null,
    created_at: "2026-01-01T00:00:00Z",
    provider: "anthropic",
    model: "claude-sonnet-5",
    usage: null,
  };
}

</storycode:copy>
<a id="f-e7bd6d898a44-TREE"></a>
### `TREE`

<storycode:copy id="f-e7bd6d898a44-TREE">
//        root
//       /    \
//      a      b        <- a rewind at `root` forked `b`
//      |      |
//      a2     b2
const TREE = [
  turn("root", null),
  turn("a", "root"),
  turn("a2", "a"),
  turn("b", "root"),
  turn("b2", "b"),
];

</storycode:copy>
<a id="f-e7bd6d898a44-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-e7bd6d898a44-expression_statement-5">
describe("live stream", () => {
  it("appends LLM tokens and legacy out/err transcript data", () => {
    let s = appendStream("", { kind: "token", data: "Hello " });
    s = appendStream(s, { kind: "token", data: "world" });
    expect(s).toBe("Hello world");
    expect(appendStream("a", { t: 1, kind: "out", data: "b" })).toBe("ab");
    expect(appendStream("a", { t: 1, kind: "err", data: "b" })).toBe("ab");
  });

  it("marks script and tool starts so a long pause reads as work", () => {
    expect(
      appendStream("x", {
        kind: "script_started",
        lang: "python",
        data: "print(1)",
      }),
    ).toContain("[running python script…]");
    expect(
      appendStream("x", {
        kind: "tool_started",
        name: "edit_doc",
        data: "<xml/>",
      }),
    ).toContain("[edit_doc…]");
  });

  it("ignores bookkeeping events instead of dumping them into the preview", () => {
    expect(appendStream("keep", { kind: "thinking" })).toBe("keep");
    expect(appendStream("keep", { kind: "turn_usage" })).toBe("keep");
    expect(
      appendStream("keep", {
        kind: "script_finished",
        result: { exit_code: 0, stdout: "s", stderr: "" },
      }),
    ).toBe("keep");
  });
});

</storycode:copy>
<a id="f-e7bd6d898a44-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-e7bd6d898a44-expression_statement-6">
describe("conversation tree", () => {
  it("shows the path from the root to the selected tip, oldest first", () => {
    expect(branchOf(TREE, "a2").map((t) => t.id)).toEqual(["root", "a", "a2"]);
    expect(branchOf(TREE, "b2").map((t) => t.id)).toEqual(["root", "b", "b2"]);
  });

  it("returns nothing for an empty or unknown tip rather than throwing", () => {
    expect(branchOf(TREE, null)).toEqual([]);
    expect(branchOf(TREE, "nope")).toEqual([]);
  });

  it("terminates on a cycle instead of hanging the dock", () => {
    const cyclic = [turn("x", "y"), turn("y", "x")];
    expect(branchOf(cyclic, "x").length).toBeLessThanOrEqual(2);
  });

  it("lists the alternatives at a fork, so the switcher can page through them", () => {
    expect(childrenOf(TREE, "root").map((t) => t.id)).toEqual(["a", "b"]);
    expect(childrenOf(TREE, null).map((t) => t.id)).toEqual(["root"]);
    expect(childrenOf(TREE, "a2")).toEqual([]);
  });

  it("switching branches lands on that branch's newest tip, not its fork point", () => {
    expect(deepestFrom(TREE, "b")).toBe("b2");
    expect(deepestFrom(TREE, "a")).toBe("a2");
    expect(deepestFrom(TREE, "a2")).toBe("a2");
  });
});

</storycode:copy>
<a id="f-e7bd6d898a44-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-e7bd6d898a44-expression_statement-7">
// The two suites below protect
// docs/guarantees/agent/the-dock-reports-spend-and-runs-the-chosen-model.md.
describe("session stats formatting", () => {
  it("compacts token counts the way the header has room for", () => {
    expect(formatTokens(0)).toBe("0");
    expect(formatTokens(950)).toBe("950");
    expect(formatTokens(12_400)).toBe("12.4k");
    expect(formatTokens(250_000)).toBe("250k");
    expect(formatTokens(3_100_000)).toBe("3.1M");
  });

  it("shows USD to four places, and a dash when the price is unknown", () => {
    expect(formatUsd(0.03419)).toBe("$0.0342");
    expect(formatUsd(0)).toBe("$0.0000");
    expect(formatUsd(null)).toBe("$—");
  });

  it("computes the cache hit rate as reads over reads-plus-uncached-input", () => {
    expect(cacheHitRate(100, 300)).toBeCloseTo(0.75);
    expect(cacheHitRate(0, 0)).toBeNull();
  });

  it("renders the one-line summary, dashing what is unknowable", () => {
    const totals: AgentTotals = {
      usd: 0.0342,
      input: 12_400,
      output: 3_100,
      cache_read: 44_000,
      cache_write: 500,
    };
    expect(statsLine(totals)).toBe("$0.0342 · in 12.4k · out 3.1k · cache 78%");
    expect(
      statsLine({
        usd: null,
        input: 0,
        output: 0,
        cache_read: 0,
        cache_write: 0,
      }),
    ).toBe("$— · in 0 · out 0 · cache —");
  });
});

</storycode:copy>
<a id="f-e7bd6d898a44-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-e7bd6d898a44-expression_statement-8">
describe("the dock's model control and stats line", () => {
  afterEach(cleanup);

  const realtime = { onRunEvent: () => () => undefined } as unknown as Realtime;

  function listing(
    overrides: Partial<AgentTurnsResponse> = {},
  ): AgentTurnsResponse {
    return {
      turns: [],
      provider: "anthropic",
      model: "claude-sonnet-5",
      totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 },
      ...overrides,
    };
  }

  function dock() {
    return render(
      <ChatDock
        docId="d1"
        realtime={realtime}
        collapsed={false}
        onToggleCollapsed={() => undefined}
        onAgentFinished={() => undefined}
      />,
    );
  }

  it("hydrates the provider select from the server's resolved default and posts the chosen values", async () => {
    vi.mocked(api.agentTurns).mockResolvedValue(listing());
    vi.mocked(api.agent).mockResolvedValue({ session_id: "s1" });
    dock();

    fireEvent.click(screen.getByRole("button", { name: "Agent settings" }));
    const select = screen.getByLabelText("Provider") as HTMLSelectElement;
    await waitFor(() => expect(select.value).toBe("anthropic"));
    // The default model stays a placeholder, not an explicit choice.
    const modelInput = screen.getByLabelText("Model") as HTMLInputElement;
    expect(modelInput.value).toBe("");
    expect(modelInput.placeholder).toBe("claude-sonnet-5");

    fireEvent.change(select, { target: { value: "openai" } });
    expect(modelInput.placeholder).toBe("gpt-5");
    fireEvent.change(modelInput, { target: { value: "gpt-5-mini" } });
    await typeResponse("do the thing");
    fireEvent.click(screen.getByText("Send"));

    await waitFor(() =>
      expect(api.agent).toHaveBeenCalledWith(
        "d1",
        "do the thing",
        null,
        "openai",
        "gpt-5-mini",
        "python",
      ),
    );
  });

  it("shows the stats line with its full-breakdown tooltip once usage exists", async () => {
    vi.mocked(api.agentTurns).mockResolvedValue(
      listing({
        totals: {
          usd: 0.0342,
          input: 12_400,
          output: 3_100,
          cache_read: 44_000,
          cache_write: 500,
        },
      }),
    );
    dock();

    const stats = await screen.findByText(
      "$0.0342 · in 12.4k · out 3.1k · cache 78%",
    );
    expect(stats.dataset.tip).toContain("cache read 44,000 tokens");
    expect(stats.dataset.tip).toContain("cache write 500 tokens");
  });

  it("shows no stats line before any turn has reported usage", async () => {
    vi.mocked(api.agentTurns).mockResolvedValue(listing());
    const { container } = dock();
    await waitFor(() => expect(api.agentTurns).toHaveBeenCalled());
    expect(container.querySelector(".chat-stats")).toBeNull();
  });
});

</storycode:copy>
<a id="f-e7bd6d898a44-import_statement-6"></a>
### `import_statement`

Moving around the tree from the keyboard, and zooming out of it

<storycode:copy id="f-e7bd6d898a44-import_statement-6">
// ---------------------------------------------------------------------------
// Moving around the tree from the keyboard, and zooming out of it
// ---------------------------------------------------------------------------
import { parseSlash, rewindFrom } from "./ChatDock";
</storycode:copy>
<a id="f-e7bd6d898a44-import_statement-7"></a>
### `import_statement`

<storycode:copy id="f-e7bd6d898a44-import_statement-7">
import { pathTo } from "./ChatTree";

</storycode:copy>
<a id="f-e7bd6d898a44-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-e7bd6d898a44-expression_statement-9">
// Guarantee: docs/guarantees/agent/a-session-is-the-conversation.md
describe("slash commands and the tree", () => {
  it("parses /rewind, /rewind N, /tree, /new, /help and nothing else", () => {
    expect(parseSlash("/rewind")).toEqual({ kind: "rewind", steps: 1 });
    expect(parseSlash("/rewind 3")).toEqual({ kind: "rewind", steps: 3 });
    expect(parseSlash("/TREE")).toEqual({ kind: "tree" });
    expect(parseSlash("/new")).toEqual({ kind: "new" });
    expect(parseSlash("/help")).toEqual({ kind: "help" });
    expect(parseSlash("/usr/bin/env is a path")).toBeNull();
    expect(parseSlash("rewind please")).toBeNull();
  });

  it("rewinds along parent pointers and stops at the root", () => {
    const turns = [
      turn("a", null),
      turn("b", "a"),
      turn("c", "b"),
      turn("d", "b"),
    ];
    expect(rewindFrom(turns, "c", 1)).toBe("b");
    expect(rewindFrom(turns, "c", 2)).toBe("a");
    expect(rewindFrom(turns, "c", 9)).toBeNull();
    expect(rewindFrom(turns, null, 1)).toBeNull();
  });

  it("the path to the tip is the branch the dock shows", () => {
    const turns = [
      turn("a", null),
      turn("b", "a"),
      turn("c", "b"),
      turn("d", "b"),
    ];
    expect([...pathTo(turns, "d")].sort()).toEqual(["a", "b", "d"]);
    expect(pathTo(turns, "c").has("d")).toBe(false);
  });
});

</storycode:copy>
<a id="f-e7bd6d898a44-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-e7bd6d898a44-expression_statement-10">
// docs/guarantees/agent/rewind-and-re-run-are-two-different-acts.md
describe("rewind and re-run are named apart", () => {
  it("parses both, and only as whole commands", () => {
    expect(parseSlash("/rerun")).toEqual({ kind: "rerun" });
    expect(parseSlash("/rewind 2")).toEqual({ kind: "rewind", steps: 2 });
    // A path or a date at the start of a sentence must not be eaten.
    expect(parseSlash("/rerun the thing")).toBeNull();
  });

  it("says what each act KEEPS, which is the only difference", () => {
    // Rewind keeps the change of mind, visible in the file forever; a re-run
    // does not, and the first attempt becomes a draft you may discard.
    expect(REWIND_TIP).toMatch(/THIS conversation/);
    expect(REWIND_TIP).toMatch(/kept/);
    expect(RERUN_TIP).toMatch(/SECOND conversation/);
    expect(RERUN_TIP).toMatch(/not kept/);
    expect(RERUN_TIP).toMatch(/gitignored/);
  });

  it("offers both in the help, with the difference stated", () => {
    expect(SLASH_HELP).toMatch(/keeping the branch/);
    expect(SLASH_HELP).toMatch(/discarding this one/);
  });
});

</storycode:copy>
<a id="f-e7bd6d898a44-expression_statement-11"></a>
### `expression_statement`

A runaway turn — a model looping mid-stream — bills tokens until somebody
pulls the cord, and "close the whole program" must never be the only cord.

<storycode:copy id="f-e7bd6d898a44-expression_statement-11">
// A runaway turn — a model looping mid-stream — bills tokens until somebody
// pulls the cord, and "close the whole program" must never be the only cord.
describe("stopping a run", () => {
  afterEach(cleanup);

  const realtime = { onRunEvent: () => () => undefined } as unknown as Realtime;

  function listing(turns: AgentTurn[] = []): AgentTurnsResponse {
    return {
      turns,
      provider: "anthropic",
      model: "claude-sonnet-5",
      totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 },
    };
  }

  function dock() {
    return render(
      <ChatDock
        docId="d1"
        realtime={realtime}
        collapsed={false}
        onToggleCollapsed={() => undefined}
        onAgentFinished={() => undefined}
      />,
    );
  }

  it("replaces Send with a Stop button while a turn runs, and it pulls the cord", async () => {
    vi.mocked(api.agentTurns).mockResolvedValue(listing());
    vi.mocked(api.agent).mockResolvedValue({ session_id: "s1" });
    vi.mocked(api.agentStop).mockResolvedValue({ stopping: "s1" });
    dock();

    await typeResponse("loop forever");
    fireEvent.click(screen.getByText("Send"));

    const stop = await screen.findByRole("button", { name: "Stop the agent" });
    // The Send button is gone — one verb at a time — and Stop is LIVE, never
    // disabled while the run is: a disabled stop is no stop at all.
    expect(screen.queryByText("Send")).toBeNull();
    expect((stop as HTMLButtonElement).disabled).toBe(false);

    fireEvent.click(stop);
    await waitFor(() => expect(api.agentStop).toHaveBeenCalledWith("d1"));
    // Pressed once: the button says so and refuses a second pull while the
    // first is in flight.
    expect(stop.textContent).toContain("Stopping…");
    expect((stop as HTMLButtonElement).disabled).toBe(true);
  });

  it("renders a stopped turn quietly — the user's own act, never a red error", async () => {
    const stopped: AgentTurn = {
      ...turn("s1", null),
      answer: null,
      status: "stopped",
      error: "stopped by you",
    };
    vi.mocked(api.agentTurns).mockResolvedValue(listing([stopped]));
    const { container } = dock();

    await screen.findByText(/Stopped by you/);
    expect(container.querySelector(".chat-error")).toBeNull();
  });
});

</storycode:copy>
<a id="f-e7bd6d898a44-typeResponse"></a>
### `typeResponse`

<storycode:copy id="f-e7bd6d898a44-typeResponse">
async function typeResponse(text: string) {
  const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
  const view = EditorView.findFromDOM(host)!;
  act(() => view.dispatch({ changes: { from: view.state.field(responseStart), to: view.state.doc.length, insert: text }, userEvent: "input.type" }));
}

</storycode:copy>
<a id="f-e7bd6d898a44-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-e7bd6d898a44-expression_statement-12">
// Guarantee: docs/guarantees/agent/unsent-responses-survive-restart.md
it("retains a rejected send and preserves a newer response when a later send is accepted", async () => {
  vi.mocked(acpApi.state).mockResolvedValue({ backend: "python", ready: true });
  vi.mocked(api.agentTurns).mockResolvedValue({ turns: [], provider: "anthropic", model: "claude-sonnet-5", totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
  vi.mocked(api.agent).mockRejectedValueOnce(new Error("send rejected"));
  const realtime = { onRunEvent: () => () => undefined } as unknown as Realtime;
  const view = render(<ChatDock docId="send-recovery" realtime={realtime} onAgentFinished={() => {}} />);
  await typeResponse("submitted text");
  await waitFor(() => expect((screen.getByRole("button", { name: "Send" }) as HTMLButtonElement).disabled).toBe(false));
  fireEvent.click(screen.getByRole("button", { name: "Send" }));
  await screen.findByText("send rejected");
  expect(screen.getByRole("textbox", { name: "Agent conversation and response" }).textContent).toContain("submitted text");
  let accept!: (result: { session_id: string }) => void;
  vi.mocked(api.agent).mockReturnValueOnce(new Promise(resolve => { accept = resolve; }));
  fireEvent.click(screen.getByRole("button", { name: "Send" }));
  await typeResponse("newer text while sending");
  await act(async () => accept({ session_id: "accepted" }));
  await screen.findByRole("button", { name: "Stop the agent" });
  expect(screen.getByRole("textbox", { name: "Agent conversation and response" }).textContent).toContain("newer text while sending");
  fireEvent.click(screen.getByRole("button", { name: "Discard response" }));
  expect(screen.getByRole("textbox", { name: "Agent conversation and response" }).textContent).not.toContain("newer text while sending");
  view.unmount(); cleanup();
});

</storycode:copy>
<a id="f-e7bd6d898a44-expression_statement-13"></a>
### `expression_statement`

<storycode:copy id="f-e7bd6d898a44-expression_statement-13">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("dispatches registered draft and Send through the actual Python conversation handler", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [] });
  vi.mocked(acpApi.state).mockResolvedValue({ backend: "python", ready: true, canRewind: true });
  vi.mocked(api.agentTurns).mockResolvedValue({ turns: [], provider: "anthropic", model: "claude-sonnet-5", totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
  vi.mocked(api.agent).mockClear().mockResolvedValue({ session_id: "real-handler-turn" });
  const view = render(<UiActionContext.Provider value={registry}><ChatDock docId="d1" realtime={{ onRunEvent: () => () => undefined } as unknown as Realtime} onAgentFinished={() => {}} /></UiActionContext.Provider>);
  try {
    await waitFor(() => expect(acpApi.state).toHaveBeenCalled());
    const parent = registry.groups().groups.find(g => g.startsWith("conversation:"))!;
    const draft = registry.observe(parent).actions.find(a => a.label === "Set conversation response draft")!;
    await act(async () => { await registry.act(registry.scope, { id: "python:parent", kind: "agent" }, { handle: draft.handle, arguments: { text: "Investigate the counterexample" }, request_id: "draft" }); });
    await waitFor(() => expect(registry.observe(parent).actions.find(a => a.label === "Send conversation response")!.enabled).toBe(true));
    const send = registry.observe(parent).actions.find(a => a.label === "Send conversation response")!;
    await act(async () => { expect(await registry.act(registry.scope, { id: "python:parent", kind: "agent" }, { handle: send.handle, arguments: {}, request_id: "send" })).toMatchObject({ outcome: { status: "queued", job: "real-handler-turn" } }); });
    expect(api.agent).toHaveBeenCalledExactlyOnceWith("d1", "Investigate the counterexample", null, "anthropic", "", "python");
    expect(screen.getByRole("button", { name: "Stop the agent" })).toBeTruthy();
  } finally { view.unmount(); }
});

</storycode:copy>
<a id="f-e7bd6d898a44-expression_statement-14"></a>
### `expression_statement`

<storycode:copy id="f-e7bd6d898a44-expression_statement-14">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("discloses actual agent settings before registered model and backend selection", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [{ id: "python", name: "Python", command: "python", args: [], available: true }, { id: "codex", name: "Codex", command: "codex-acp", args: [], available: true }] });
  vi.mocked(acpApi.state).mockResolvedValue({ backend: "python", ready: true, canRewind: true });
  vi.mocked(acpApi.connect).mockClear().mockResolvedValue({ backend: "codex", ready: true });
  vi.mocked(api.agentTurns).mockResolvedValue({ turns: [], provider: "anthropic", model: "claude-sonnet-5", totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
  vi.mocked(api.agent).mockClear();
  const view = render(<UiActionContext.Provider value={registry}><ChatDock docId="d1" realtime={{ onRunEvent: () => () => undefined } as unknown as Realtime} onAgentFinished={() => {}} /></UiActionContext.Provider>);
  let n = 0;
  async function invoke(parent: string, label: string, args: Record<string, string | boolean>) {
    const action = registry.observe(parent).actions.find(a => a.label === label)!;
    await act(async () => { await registry.act(registry.scope, { id: "python:parent", kind: "agent" }, { handle: action.handle, arguments: args, request_id: String(++n) }); });
  }
  try {
    expect(registry.groups().groups.some(g => g.startsWith("conversation-model:"))).toBe(false);
    await invoke("feature-settings", "Set Agent settings visibility", { open: true });
    await screen.findByRole("option", { name: "Codex (ACP)" });
    const model = registry.groups().groups.find(g => g.startsWith("conversation-model:"))!;
    await invoke(model, "Set conversation model", { model: "chosen-model" });
    expect((screen.getByLabelText("Model") as HTMLInputElement).value).toBe("chosen-model");
    const picker = registry.groups().groups.find(g => g.startsWith("agent-picker:"))!;
    await invoke(picker, "Select conversation agent", { backend: "codex" });
    await waitFor(() => expect(acpApi.connect).toHaveBeenCalledWith("d1", "codex", undefined));
    expect(localStorage.getItem("hickory.agent")).toBe("codex");
    expect(registry.observe(model).actions).toEqual([]);
    await invoke("feature-settings", "Set Agent settings visibility", { open: false });
    expect(registry.observe(picker).actions).toEqual([]);
    expect(api.agent).not.toHaveBeenCalled();
  } finally { view.unmount(); }
});

</storycode:copy>
<a id="f-e7bd6d898a44-expression_statement-15"></a>
### `expression_statement`

<storycode:copy id="f-e7bd6d898a44-expression_statement-15">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("invalidates model and backend handles when the conversation document changes", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [{ id: "python", name: "Python", command: "python", args: [], available: true }, { id: "codex", name: "Codex", command: "codex-acp", args: [], available: true }] });
  vi.mocked(acpApi.state).mockResolvedValue({ backend: "python", ready: true });
  vi.mocked(api.agentTurns).mockResolvedValue({ turns: [], provider: "anthropic", model: "claude-sonnet-5", totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
  const realtime = { onRunEvent: () => () => undefined } as unknown as Realtime;
  const content = (docId: string) => <UiActionContext.Provider value={registry}><ChatDock docId={docId} realtime={realtime} onAgentFinished={() => {}} /></UiActionContext.Provider>;
  const view = render(content("first"));
  try {
    fireEvent.click(screen.getByRole("button", { name: "Agent settings" }));
    await screen.findByRole("option", { name: "Codex (ACP)" });
    const picker = registry.groups().groups.find(g => g.startsWith("agent-picker:"))!;
    const model = registry.groups().groups.find(g => g.startsWith("conversation-model:"))!;
    const agentAction = registry.observe(picker).actions.find(a => a.label === "Select conversation agent")!;
    const modelAction = registry.observe(model).actions.find(a => a.label === "Set conversation model")!;
    view.rerender(content("second"));
    await act(async () => {});
    expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: agentAction.handle, arguments: { backend: "codex" }, request_id: "old-backend" })).toMatchObject({ status: "stale" });
    expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: modelAction.handle, arguments: { model: "other" }, request_id: "old-model" })).toMatchObject({ status: "stale" });
    expect(localStorage.getItem("hickory.agent")).toBeNull();
    expect((screen.getByLabelText("Model") as HTMLInputElement).value).toBe("");
  } finally { view.unmount(); }
});
</storycode:copy>

### Assemble `apps/web/src/components/ChatDock.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ChatDock.test.tsx"><storycode:paste select="#f-e7bd6d898a44-expression_statement"/><storycode:paste select="#f-e7bd6d898a44-expression_statement-2"/><storycode:paste select="#f-e7bd6d898a44-import_statement"/><storycode:paste select="#f-e7bd6d898a44-import_statement-2"/><storycode:paste select="#f-e7bd6d898a44-import_statement-3"/><storycode:paste select="#f-e7bd6d898a44-import_statement-4"/><storycode:paste select="#f-e7bd6d898a44-import_statement-5"/><storycode:paste select="#f-e7bd6d898a44-expression_statement-3"/><storycode:paste select="#f-e7bd6d898a44-expression_statement-4"/><storycode:paste select="#f-e7bd6d898a44-turn"/><storycode:paste select="#f-e7bd6d898a44-TREE"/><storycode:paste select="#f-e7bd6d898a44-expression_statement-5"/><storycode:paste select="#f-e7bd6d898a44-expression_statement-6"/><storycode:paste select="#f-e7bd6d898a44-expression_statement-7"/><storycode:paste select="#f-e7bd6d898a44-expression_statement-8"/><storycode:paste select="#f-e7bd6d898a44-import_statement-6"/><storycode:paste select="#f-e7bd6d898a44-import_statement-7"/><storycode:paste select="#f-e7bd6d898a44-expression_statement-9"/><storycode:paste select="#f-e7bd6d898a44-expression_statement-10"/><storycode:paste select="#f-e7bd6d898a44-expression_statement-11"/><storycode:paste select="#f-e7bd6d898a44-typeResponse"/><storycode:paste select="#f-e7bd6d898a44-expression_statement-12"/><storycode:paste select="#f-e7bd6d898a44-expression_statement-13"/><storycode:paste select="#f-e7bd6d898a44-expression_statement-14"/><storycode:paste select="#f-e7bd6d898a44-expression_statement-15"/></storycode:file>


</storycode:doc>
